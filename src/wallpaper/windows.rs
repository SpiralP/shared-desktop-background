use std::{iter::once, os::windows::ffi::OsStrExt, path::Path};

use anyhow::Result;
use winapi::{
    shared::minwindef::TRUE,
    um::{
        winnt::PVOID,
        winuser::{
            SystemParametersInfoW, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE, SPI_SETDESKWALLPAPER, *,
        },
    },
};

pub fn set(path: &Path) -> Result<()> {
    let mut wide: Vec<u16> = path.as_os_str().encode_wide().chain(once(0)).collect();
    if unsafe {
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            wide.as_mut_ptr() as PVOID,
            SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
        )
    } != TRUE
    {
        return Err(std::io::Error::last_os_error().into());
    }

    Ok(())
}

pub fn get_screen_height() -> u16 {
    unsafe { GetSystemMetrics(SM_CYSCREEN) as u16 }
}
