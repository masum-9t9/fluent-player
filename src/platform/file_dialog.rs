use std::path::PathBuf;

use windows_sys::Win32::{
    System::Com::CoTaskMemFree,
    UI::{
        Controls::Dialogs::{
            GetOpenFileNameW, OFN_ALLOWMULTISELECT, OFN_EXPLORER, OFN_FILEMUSTEXIST,
            OFN_NOCHANGEDIR, OFN_PATHMUSTEXIST, OPENFILENAMEW,
        },
        Shell::{
            BIF_NEWDIALOGSTYLE, BIF_RETURNONLYFSDIRS, BROWSEINFOW, SHBrowseForFolderW,
            SHGetPathFromIDListW,
        },
    },
};

pub fn pick_media_file() -> Option<PathBuf> {
    pick_media_files()?.into_iter().next()
}

pub fn pick_media_files() -> Option<Vec<PathBuf>> {
    let filter: Vec<u16> = "Video files\0*.mp4;*.mkv;*.webm;*.mov;*.avi;*.wmv;*.m4v;*.mpg;*.mpeg;*.ts;*.m2ts\0All files\0*.*\0\0"
        .encode_utf16()
        .collect();
    let title: Vec<u16> = "Open a video".encode_utf16().chain([0]).collect();
    let mut filename = vec![0u16; 32_768];
    let mut dialog = OPENFILENAMEW::default();
    dialog.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
    dialog.lpstrFilter = filter.as_ptr();
    dialog.lpstrFile = filename.as_mut_ptr();
    dialog.nMaxFile = filename.len() as u32;
    dialog.lpstrTitle = title.as_ptr();
    dialog.Flags = OFN_EXPLORER
        | OFN_ALLOWMULTISELECT
        | OFN_FILEMUSTEXIST
        | OFN_PATHMUSTEXIST
        | OFN_NOCHANGEDIR;

    let selected = unsafe { GetOpenFileNameW(&mut dialog) } != 0;
    if !selected {
        return None;
    }

    let fields = filename
        .split(|unit| *unit == 0)
        .take_while(|field| !field.is_empty())
        .collect::<Vec<_>>();
    let first = fields.first()?;
    if fields.len() == 1 {
        return Some(vec![PathBuf::from(String::from_utf16_lossy(first))]);
    }

    let directory = PathBuf::from(String::from_utf16_lossy(first));
    let paths = fields[1..]
        .iter()
        .map(|name| directory.join(String::from_utf16_lossy(name).to_string()))
        .collect::<Vec<_>>();
    (!paths.is_empty()).then_some(paths)
}

pub fn pick_media_folder() -> Option<PathBuf> {
    let title: Vec<u16> = "Choose a media folder".encode_utf16().chain([0]).collect();
    let mut display_name = vec![0u16; 32_768];
    let info = BROWSEINFOW {
        pszDisplayName: display_name.as_mut_ptr(),
        lpszTitle: title.as_ptr(),
        ulFlags: BIF_NEWDIALOGSTYLE | BIF_RETURNONLYFSDIRS,
        ..Default::default()
    };

    let item = unsafe { SHBrowseForFolderW(&info) };
    if item.is_null() {
        return None;
    }

    let mut path = vec![0u16; 32_768];
    let found = unsafe { SHGetPathFromIDListW(item, path.as_mut_ptr()) } != 0;
    unsafe { CoTaskMemFree(item.cast()) };
    if !found {
        return None;
    }

    let length = path.iter().position(|unit| *unit == 0)?;
    Some(PathBuf::from(String::from_utf16_lossy(&path[..length])))
}
