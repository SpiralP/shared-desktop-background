pub mod config;
pub mod drive;
pub mod wallpaper;

use std::{env::args, time::Duration};

use anyhow::{bail, ensure, Context, Result};
use futures::StreamExt;
use rand::{seq::SliceRandom, thread_rng};
use tokio::{fs::File, io::AsyncWriteExt, time::MissedTickBehavior};

use crate::{
    config::{get_config_dir, get_seen_ids, set_file_seen},
    drive::Drive,
};

// 1 hour
const INTERVAL: Duration = Duration::from_secs(60 * 60);

#[tokio::main]
async fn main() -> Result<()> {
    let folder_name = args().nth(1).context("missing folder name argument")?;

    let drive = Drive::new(include_bytes!("../service-account.json")).await?;

    let mut interval = tokio::time::interval(INTERVAL);
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

    let mut first = true;
    loop {
        interval.tick().await;

        if let Err(e) = check_wallpaper(&folder_name, &drive, &mut first).await {
            println!("{:?}", e);
        }
    }
}

pub async fn check_wallpaper(folder_name: &str, drive: &Drive, first: &mut bool) -> Result<()> {
    let file_ids = drive.list_in_folder(folder_name).await?;

    let file_id = select_file(&file_ids, first).await?;

    set_wallpaper(file_id, drive).await?;
    *first = false;

    Ok(())
}

pub async fn select_file<'a>(file_ids: &'a [String], first: &mut bool) -> Result<&'a String> {
    let seen_files = get_seen_ids().await?;

    // show files we haven't seen before, first
    if let Some(file_id) = file_ids
        .iter()
        .find(|file_id| !seen_files.contains(*file_id))
    {
        return Ok(file_id);
    }

    if *first {
        let mut rng = thread_rng();
        let file_id = file_ids.choose(&mut rng).context("file_ids empty")?;
        return Ok(file_id);
    }

    bail!("no new files");
}

pub async fn set_wallpaper(file_id: &str, drive: &Drive) -> Result<()> {
    println!("{:?}", file_id);

    let response = drive.download_file(file_id).await?;
    ensure!(response.status().is_success());

    let content_type = response
        .headers()
        .get("content-type")
        .context("content-type header missing")?
        .to_str()?;

    let extension = match content_type {
        "image/png" => "png",
        "image/jpeg" | "image/jpg" => "jpg",
        other => bail!("weird content-type {:?}", other),
    };

    let file_path = get_config_dir()
        .await?
        .join(format!("background.{}", extension));

    {
        let mut f = File::create(&file_path).await?;
        let mut body = response.into_body();
        while let Some(result) = body.next().await {
            let chunk = result?;
            f.write_all(&chunk).await?;
        }
    }
    wallpaper::set(&file_path)?;

    set_file_seen(file_id).await?;

    Ok(())
}
