use std::{
    ffi::{CString, c_char, c_void},
    path::{Path, PathBuf},
};

use libloading::Library;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows_sys::Win32::{
    Foundation::{HWND, RECT},
    UI::WindowsAndMessaging::{
        CreateWindowExW, DestroyWindow, GetClientRect, HWND_TOP, SW_HIDE, SW_SHOW, SWP_NOACTIVATE,
        SetWindowPos, ShowWindow, WS_CHILD, WS_CLIPCHILDREN, WS_CLIPSIBLINGS, WS_VISIBLE,
    },
};

type Create = unsafe extern "C" fn() -> *mut c_void;
type SetOption = unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char) -> i32;
type SetPropertyString = unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char) -> i32;
type Initialize = unsafe extern "C" fn(*mut c_void) -> i32;
type Command = unsafe extern "C" fn(*mut c_void, *const *const c_char) -> i32;
type GetProperty = unsafe extern "C" fn(*mut c_void, *const c_char, i32, *mut c_void) -> i32;
type TerminateDestroy = unsafe extern "C" fn(*mut c_void);
type ErrorString = unsafe extern "C" fn(i32) -> *const c_char;

const VIDEO_TOP_INSET: f32 = 38.0;
const VIDEO_BOTTOM_INSET: f32 = 64.0;
const MPV_FORMAT_DOUBLE: i32 = 5;

pub struct MpvPlayer {
    _library: Library,
    handle: *mut c_void,
    command: Command,
    set_property_string: SetPropertyString,
    get_property: GetProperty,
    terminate_destroy: TerminateDestroy,
    error_string: ErrorString,
    surface: HWND,
    hardware_decoding: bool,
    ab_loop_stage: u8,
}

impl MpvPlayer {
    pub fn new(window: &slint::Window, scale_factor: f32) -> anyhow::Result<Self> {
        let library_path = locate_mpv().ok_or_else(|| {
            anyhow::anyhow!("mpv runtime is missing. Put libmpv-2.dll in the app folder or third_party/mpv, or install a Windows mpv build.")
        })?;
        let library = unsafe { Library::new(&library_path) }.map_err(|error| {
            anyhow::anyhow!("Could not load {}: {error}", library_path.display())
        })?;

        let create: Create = unsafe { *library.get(b"mpv_create\0")? };
        let set_option: SetOption = unsafe { *library.get(b"mpv_set_option_string\0")? };
        let initialize: Initialize = unsafe { *library.get(b"mpv_initialize\0")? };
        let command: Command = unsafe { *library.get(b"mpv_command\0")? };
        let set_property_string: SetPropertyString =
            unsafe { *library.get(b"mpv_set_property_string\0")? };
        let get_property: GetProperty = unsafe { *library.get(b"mpv_get_property\0")? };
        let terminate_destroy: TerminateDestroy =
            unsafe { *library.get(b"mpv_terminate_destroy\0")? };
        let error_string: ErrorString = unsafe { *library.get(b"mpv_error_string\0")? };

        let raw = window.window_handle().window_handle()?.as_raw();
        let RawWindowHandle::Win32(native) = raw else {
            anyhow::bail!("The embedded video surface is only available on Windows.");
        };
        let parent = native.hwnd.get() as HWND;
        let mut client = RECT::default();
        anyhow::ensure!(
            unsafe { GetClientRect(parent, &mut client) } != 0,
            "Could not read the player window size"
        );
        let top = (VIDEO_TOP_INSET * scale_factor).round() as i32;
        let bottom = (VIDEO_BOTTOM_INSET * scale_factor).round() as i32;
        let width = (client.right - client.left - 10).max(1);
        let height = (client.bottom - client.top - top - bottom).max(1);
        let class_name: Vec<u16> = "STATIC\0".encode_utf16().collect();
        let surface = unsafe {
            CreateWindowExW(
                0,
                class_name.as_ptr(),
                std::ptr::null(),
                WS_CHILD | WS_CLIPCHILDREN | WS_CLIPSIBLINGS | WS_VISIBLE,
                5,
                top,
                width,
                height,
                parent,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null(),
            )
        };
        anyhow::ensure!(
            !surface.is_null(),
            "Could not create the embedded video surface"
        );

        let handle = unsafe { create() };
        if handle.is_null() {
            unsafe { DestroyWindow(surface) };
            anyhow::bail!("mpv could not create a player instance");
        }

        let wid = CString::new(format!("{}", surface as usize as u32))?;

        for (name, value) in [
            ("vo", "gpu-next"),
            ("gpu-context", "d3d11"),
            ("hwdec", "auto-safe"),
            ("wid", wid.to_str()?),
            ("keep-open", "yes"),
            ("osd-level", "0"),
            ("osc", "no"),
            ("input-default-bindings", "no"),
            ("volume-max", "200"),
            ("alang", ""),
            ("slang", ""),
        ] {
            let name = CString::new(name)?;
            let value = CString::new(value)?;
            let result = unsafe { set_option(handle, name.as_ptr(), value.as_ptr()) };
            if result < 0 {
                unsafe { terminate_destroy(handle) };
                unsafe { DestroyWindow(surface) };
                anyhow::bail!("mpv rejected startup option ({result})");
            }
        }

        let result = unsafe { initialize(handle) };
        if result < 0 {
            let message = unsafe { std::ffi::CStr::from_ptr(error_string(result)) }
                .to_string_lossy()
                .into_owned();
            unsafe { terminate_destroy(handle) };
            unsafe { DestroyWindow(surface) };
            anyhow::bail!("mpv initialization failed: {message}");
        }

        Ok(Self {
            _library: library,
            handle,
            command,
            set_property_string,
            get_property,
            terminate_destroy,
            error_string,
            surface,
            hardware_decoding: true,
            ab_loop_stage: 0,
        })
    }

    pub fn show_surface(&mut self, visible: bool) {
        unsafe { ShowWindow(self.surface, if visible { SW_SHOW } else { SW_HIDE }) };
    }

    pub fn resize_surface(&mut self, window: &slint::Window, scale_factor: f32) {
        let mut client = RECT::default();
        let hwnd = match window
            .window_handle()
            .window_handle()
            .map(|handle| handle.as_raw())
        {
            Ok(RawWindowHandle::Win32(handle)) => handle.hwnd.get() as HWND,
            _ => return,
        };
        if unsafe { GetClientRect(hwnd, &mut client) } == 0 {
            return;
        }
        let top = (VIDEO_TOP_INSET * scale_factor).round() as i32;
        let bottom = (VIDEO_BOTTOM_INSET * scale_factor).round() as i32;
        let width = (client.right - client.left - 10).max(1);
        let height = (client.bottom - client.top - top - bottom).max(1);
        unsafe {
            SetWindowPos(
                self.surface,
                HWND_TOP,
                5,
                top,
                width,
                height,
                SWP_NOACTIVATE,
            );
        }
    }

    pub fn open(&mut self, path: &Path) -> anyhow::Result<()> {
        let path = CString::new(path.to_string_lossy().as_bytes())?;
        let args = [CString::new("loadfile")?, path, CString::new("replace")?];
        let pointers = [
            args[0].as_ptr(),
            args[1].as_ptr(),
            args[2].as_ptr(),
            std::ptr::null(),
        ];
        let result = unsafe { (self.command)(self.handle, pointers.as_ptr()) };
        if result < 0 {
            let message = unsafe { std::ffi::CStr::from_ptr((self.error_string)(result)) }
                .to_string_lossy()
                .into_owned();
            anyhow::bail!("mpv could not open this video: {message}");
        }
        Ok(())
    }

    /// Add a media item after the current playlist entry and keep it queued.
    pub fn append(&mut self, path: &Path) -> anyhow::Result<()> {
        let path = CString::new(path.to_string_lossy().as_bytes())?;
        self.command_args_owned(&[
            CString::new("loadfile")?,
            path,
            CString::new("append")?,
        ])
    }

    pub fn play_next(&mut self) -> anyhow::Result<()> {
        self.command_args(&["playlist-next"])
    }

    pub fn play_previous(&mut self) -> anyhow::Result<()> {
        self.command_args(&["playlist-prev"])
    }

    pub fn step_frame(&mut self) -> anyhow::Result<()> {
        self.command_args(&["frame-step"])
    }

    pub fn toggle_ab_loop(&mut self) -> anyhow::Result<()> {
        let position = self.playback_times().0;
        match self.ab_loop_stage {
            0 => {
                self.set_property("ab-loop-a", &format!("{position:.3}"))?;
                self.set_property("ab-loop-b", "no")?;
                self.ab_loop_stage = 1;
            }
            1 => {
                self.set_property("ab-loop-b", &format!("{position:.3}"))?;
                self.ab_loop_stage = 2;
            }
            _ => {
                self.set_property("ab-loop-a", "no")?;
                self.set_property("ab-loop-b", "no")?;
                self.ab_loop_stage = 0;
            }
        }
        Ok(())
    }

    pub fn play_index(&mut self, index: usize) -> anyhow::Result<()> {
        self.set_property("playlist-pos", &index.to_string())
    }

    pub fn playlist_position(&self) -> Option<usize> {
        let name = CString::new("playlist-pos").ok()?;
        let mut value = -1i64;
        let result = unsafe {
            (self.get_property)(
                self.handle,
                name.as_ptr(),
                4,
                (&mut value as *mut i64).cast(),
            )
        };
        (result >= 0 && value >= 0).then_some(value as usize)
    }

    pub fn toggle_pause(&mut self) -> anyhow::Result<()> {
        self.command_args(&["cycle", "pause"])
    }

    pub fn resume(&mut self) -> anyhow::Result<()> {
        self.command_args(&["set", "pause", "no"])
    }

    pub fn screenshot(&mut self, path: &Path) -> anyhow::Result<()> {
        let path = CString::new(path.to_string_lossy().as_bytes())?;
        self.command_args_owned(&[
            CString::new("screenshot-to-file")?,
            path,
            CString::new("video")?,
        ])
    }

    pub fn seek_to(&mut self, seconds: f64) -> anyhow::Result<()> {
        let seconds = CString::new(format!("{seconds:.3}"))?;
        self.command_args_owned(&[CString::new("seek")?, seconds, CString::new("absolute")?])
    }

    pub fn seek_relative(&mut self, seconds: f64) -> anyhow::Result<()> {
        let seconds = CString::new(format!("{seconds:.3}"))?;
        self.command_args_owned(&[CString::new("seek")?, seconds, CString::new("relative")?])
    }

    pub fn toggle_mute(&mut self) -> anyhow::Result<()> {
        self.command_args(&["cycle", "mute"])
    }

    pub fn toggle_subtitles(&mut self) -> anyhow::Result<()> {
        self.command_args(&["cycle", "sub-visibility"])
    }

    pub fn toggle_repeat(&mut self) -> anyhow::Result<()> {
        self.command_args(&["cycle-values", "loop-file", "inf", "no"])
    }

    pub fn toggle_playlist_repeat(&mut self) -> anyhow::Result<()> {
        self.command_args(&["cycle-values", "loop-playlist", "inf", "no"])
    }

    pub fn cycle_audio_track(&mut self) -> anyhow::Result<()> {
        self.command_args(&["cycle", "aid"])
    }

    pub fn cycle_subtitle_track(&mut self) -> anyhow::Result<()> {
        self.command_args(&["cycle", "sid"])
    }

    pub fn volume(&self) -> f64 {
        self.read_number("volume").clamp(0.0, 200.0)
    }

    pub fn set_volume(&mut self, value: f64) -> anyhow::Result<()> {
        let value = if value.is_finite() { value } else { 100.0 }.clamp(0.0, 200.0);
        self.set_property("volume", &format!("{value:.0}"))
    }

    pub fn speed(&self) -> f64 {
        self.read_number("speed").clamp(0.25, 4.0)
    }

    pub fn set_speed(&mut self, value: f64) -> anyhow::Result<()> {
        let value = if value.is_finite() { value } else { 1.0 }.clamp(0.25, 4.0);
        self.set_property("speed", &format!("{value:.2}"))
    }

    pub fn toggle_hardware_decoding(&mut self) -> anyhow::Result<bool> {
        let enabled = !self.hardware_decoding;
        self.set_property("hwdec", if enabled { "auto-safe" } else { "no" })?;
        self.hardware_decoding = enabled;
        Ok(enabled)
    }

    pub fn adjust_video_property(&mut self, property: &str, delta: f64) -> anyhow::Result<()> {
        const ALLOWED: [&str; 4] = ["brightness", "contrast", "saturation", "gamma"];
        anyhow::ensure!(ALLOWED.contains(&property), "Unsupported video adjustment");
        let delta = if delta.is_finite() { delta } else { 0.0 };
        let argument = CString::new(format!("{delta:.1}"))?;
        self.command_args_owned(&[CString::new("add")?, CString::new(property)?, argument])
    }

    pub fn reset_video_adjustments(&mut self) -> anyhow::Result<()> {
        for property in ["brightness", "contrast", "saturation", "gamma"] {
            self.set_property(property, "0")?;
        }
        Ok(())
    }

    pub fn playback_times(&self) -> (f64, f64) {
        fn read(get_property: GetProperty, handle: *mut c_void, name: &str) -> f64 {
            let Ok(name) = CString::new(name) else {
                return 0.0;
            };
            let mut value = 0.0f64;
            let result = unsafe {
                get_property(
                    handle,
                    name.as_ptr(),
                    MPV_FORMAT_DOUBLE,
                    (&mut value as *mut f64).cast(),
                )
            };
            if result >= 0 && value.is_finite() {
                value.max(0.0)
            } else {
                0.0
            }
        }
        (
            read(self.get_property, self.handle, "time-pos"),
            read(self.get_property, self.handle, "duration"),
        )
    }

    fn read_number(&self, property: &str) -> f64 {
        let Ok(name) = CString::new(property) else {
            return 0.0;
        };
        let mut value = 0.0f64;
        let result = unsafe {
            (self.get_property)(
                self.handle,
                name.as_ptr(),
                MPV_FORMAT_DOUBLE,
                (&mut value as *mut f64).cast(),
            )
        };
        if result >= 0 && value.is_finite() { value } else { 0.0 }
    }

    fn set_property(&mut self, property: &str, value: &str) -> anyhow::Result<()> {
        let property = CString::new(property)?;
        let value = CString::new(value)?;
        let result = unsafe {
            (self.set_property_string)(self.handle, property.as_ptr(), value.as_ptr())
        };
        self.check_result(result, "mpv rejected the playback setting")
    }

    fn check_result(&self, result: i32, context: &str) -> anyhow::Result<()> {
        if result < 0 {
            let message = unsafe { std::ffi::CStr::from_ptr((self.error_string)(result)) }
                .to_string_lossy()
                .into_owned();
            anyhow::bail!("{context}: {message}");
        }
        Ok(())
    }

    fn command_args(&mut self, args: &[&str]) -> anyhow::Result<()> {
        let strings = args
            .iter()
            .map(|arg| CString::new(*arg))
            .collect::<Result<Vec<_>, _>>()?;
        self.command_args_owned(&strings)
    }

    fn command_args_owned(&mut self, strings: &[CString]) -> anyhow::Result<()> {
        let mut pointers: Vec<*const c_char> = strings.iter().map(|arg| arg.as_ptr()).collect();
        pointers.push(std::ptr::null());
        let result = unsafe { (self.command)(self.handle, pointers.as_ptr()) };
        if result < 0 {
            let message = unsafe { std::ffi::CStr::from_ptr((self.error_string)(result)) }
                .to_string_lossy()
                .into_owned();
            anyhow::bail!("mpv could not toggle playback: {message}");
        }
        Ok(())
    }
}

impl Drop for MpvPlayer {
    fn drop(&mut self) {
        unsafe { (self.terminate_destroy)(self.handle) }
        unsafe { DestroyWindow(self.surface) };
    }
}

fn locate_mpv() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(project_dir) = option_env!("CARGO_MANIFEST_DIR") {
        candidates.push(
            PathBuf::from(project_dir)
                .join("third_party")
                .join("mpv")
                .join("libmpv-2.dll"),
        );
        candidates.push(
            PathBuf::from(project_dir)
                .join("third_party")
                .join("mpv")
                .join("mpv-2.dll"),
        );
    }
    if let Ok(current_dir) = std::env::current_dir() {
        candidates.push(
            current_dir
                .join("third_party")
                .join("mpv")
                .join("libmpv-2.dll"),
        );
        candidates.push(
            current_dir
                .join("third_party")
                .join("mpv")
                .join("mpv-2.dll"),
        );
    }
    if let Ok(executable) = std::env::current_exe() {
        if let Some(folder) = executable.parent() {
            candidates.push(folder.join("mpv-2.dll"));
            candidates.push(folder.join("libmpv-2.dll"));
            candidates.push(folder.join("mpv").join("libmpv-2.dll"));
            candidates.push(folder.join("third_party").join("mpv").join("libmpv-2.dll"));
            candidates.push(folder.join("third_party").join("mpv").join("mpv-2.dll"));
        }
    }
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        candidates.push(
            PathBuf::from(&local_app_data)
                .join("Programs")
                .join("mpv")
                .join("libmpv-2.dll"),
        );
    }
    if let Some(program_files) = std::env::var_os("ProgramFiles") {
        candidates.push(
            PathBuf::from(program_files)
                .join("mpv")
                .join("libmpv-2.dll"),
        );
    }
    if let Some(path) = std::env::var_os("PATH") {
        candidates.extend(std::env::split_paths(&path).map(|folder| folder.join("libmpv-2.dll")));
    }
    candidates.into_iter().find(|path| path.is_file())
}
