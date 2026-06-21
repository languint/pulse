use std::process::Command;

use gpui::App;

use crate::data::DataPaths;

/// Opens the user themes directory in the system file manager.
pub fn open_themes_folder(cx: &App) {
    let path = cx.global::<DataPaths>().themes_dir();
    let _ = std::fs::create_dir_all(&path);

    #[cfg(windows)]
    {
        let _ = Command::new("explorer").arg(path).spawn();
    }

    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("open").arg(&path).spawn();
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let _ = Command::new("xdg-open").arg(&path).spawn();
    }
}
