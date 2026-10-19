#[cfg(windows)]
mod file_dialog;

#[cfg(windows)]
pub use file_dialog::{pick_media_file, pick_media_files, pick_media_folder};

#[cfg(not(windows))]
pub fn pick_media_file() -> Option<std::path::PathBuf> {
    None
}

#[cfg(not(windows))]
pub fn pick_media_files() -> Option<Vec<std::path::PathBuf>> {
    None
}

#[cfg(not(windows))]
pub fn pick_media_folder() -> Option<std::path::PathBuf> {
    None
}
