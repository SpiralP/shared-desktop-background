use std::{env, ffi::OsString, fs, path::Path, process::Command, ptr};

use anyhow::{ensure, Context, Result};
use x11::xlib::*;

pub fn set(path: &Path) -> Result<()> {
    if let Ok(desktop_session) = env::var("DESKTOP_SESSION") {
        if desktop_session.contains("plasma") {
            let secondary_path = path.parent().unwrap().join(format!(
                "{}2.{}",
                path.file_stem().unwrap().to_string_lossy(),
                path.extension().unwrap().to_string_lossy()
            ));

            let mut file_url = OsString::from("file://");
            if secondary_path.exists() {
                fs::remove_file(&secondary_path)?;
                file_url.push(path);
            } else {
                fs::rename(path, &secondary_path)?;
                file_url.push(&secondary_path);
            }

            ensure!(
                Command::new("qdbus")
                    .args([
                        "org.kde.plasmashell",
                        "/PlasmaShell",
                        "org.kde.PlasmaShell.evaluateScript",
                        &format!(
                            r#"
                desktops().forEach((d) => {{
                  d.wallpaperPlugin = "org.kde.image";
                  d.currentConfigGroup = ["Wallpaper", "org.kde.image", "General"];
                  d.writeConfig("Image", "{}");
                  d.reloadConfig();
                }});
              "#,
                            file_url.to_str().context("to_str")?
                        ),
                    ])
                    .status()?
                    .success(),
                "command failed"
            );

            return Ok(());
        }
    }

    // gsettings set org.gnome.desktop.background picture-options scaled

    let mut file_url = OsString::from("file://");
    file_url.push(path);

    ensure!(
        Command::new("gsettings")
            .args([
                "set",
                "org.gnome.desktop.background",
                "picture-uri",
                file_url.to_str().context("to_str")?,
            ])
            .status()?
            .success(),
        "command failed"
    );

    Ok(())
}

pub fn get_screen_height() -> u16 {
    unsafe {
        let display = XOpenDisplay(ptr::null());
        let screen = XDefaultScreen(display);
        let height = XDisplayHeight(display, screen) as u16;
        XCloseDisplay(display);

        height
    }
}
