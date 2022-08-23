use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use tokio::fs;

pub async fn get_config_dir() -> Result<PathBuf> {
    let config_dir =
        create_dir_if_missing(dirs::config_dir().context("dirs::config_dir() None")?).await?;
    let config_dir = create_dir_if_missing(config_dir.join(env!("CARGO_PKG_NAME"))).await?;
    Ok(config_dir)
}

pub async fn get_seen_ids_path() -> Result<PathBuf> {
    Ok(get_config_dir().await?.join("seen-ids.txt"))
}

pub async fn create_dir_if_missing<P: AsRef<Path>>(dir: P) -> Result<PathBuf> {
    let dir = dir.as_ref();

    if !dir.is_dir() {
        println!("creating new directory {:?}", dir);
        fs::create_dir(dir)
            .await
            .with_context(|| format!("creating {:?}", dir))?;
    }

    Ok(dir.to_path_buf())
}

pub async fn get_seen_ids() -> Result<HashSet<String>> {
    let seen_ids_path = get_seen_ids_path().await?;

    if !fs::metadata(&seen_ids_path)
        .await
        .map(|metadata| metadata.is_file())
        .unwrap_or(false)
    {
        return Ok(HashSet::new());
    }

    let contents = fs::read_to_string(&seen_ids_path).await?;
    let seen_ids = contents
        .split('\n')
        .filter(|id| !id.is_empty())
        .map(|id| id.to_owned())
        .collect::<HashSet<_>>();

    Ok(seen_ids)
}

pub async fn set_file_seen(file_id: &str) -> Result<()> {
    let mut seen_ids = get_seen_ids().await?;
    seen_ids.insert(file_id.to_owned());

    let seen_ids_path = get_seen_ids_path().await?;

    let mut contents = seen_ids.into_iter().collect::<Vec<_>>();
    contents.sort();
    let contents = contents.join("\n");
    fs::write(&seen_ids_path, contents).await?;

    Ok(())
}
