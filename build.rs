use std::{
    env,
    path::PathBuf,
    process::Command,
};

fn main() {
    // Compiles ui/main_window.slint into generated Rust code.
    // The generated code is included in src/ui.rs via slint::include_modules!().
    slint_build::compile("ui/main_window.slint").expect("Slint UI failed to compile");

    println!("cargo:rerun-if-changed=ui/assets/fluent-player.jpg");
    println!("cargo:rerun-if-changed=assets/windows/fluent-player.ico");
    println!("cargo:rerun-if-changed=assets/windows/fluent-player.rc");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_windows_resources();
    }
}

fn embed_windows_resources() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let resource_script = manifest_dir.join("assets/windows/fluent-player.rc");
    let output_dir = PathBuf::from(env::var_os("OUT_DIR").expect("build output directory"));
    let resource_file = output_dir.join("fluent-player.res");
    let resource_compiler = find_resource_compiler().unwrap_or_else(|| {
        panic!("Windows SDK rc.exe is required to embed the Fluent Player app icon")
    });

    let status = Command::new(&resource_compiler)
        .current_dir(&manifest_dir)
        .arg("/nologo")
        .arg("/fo")
        .arg(&resource_file)
        .arg(&resource_script)
        .status()
        .unwrap_or_else(|error| panic!("Could not run {}: {error}", resource_compiler.display()));
    assert!(status.success(), "Windows resource compiler failed");

    // The MSVC linker accepts .res files directly as native resources.
    println!("cargo:rustc-link-arg={}", resource_file.display());
}

fn find_resource_compiler() -> Option<PathBuf> {
    if let Some(path) = find_on_path("rc.exe") {
        return Some(path);
    }

    let program_files = env::var_os("ProgramFiles(x86)")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Program Files (x86)"));
    let sdk_bins = program_files.join("Windows Kits/10/bin");
    let mut versions = std::fs::read_dir(sdk_bins)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    versions.sort_by(|left, right| right.file_name().cmp(&left.file_name()));
    versions
        .into_iter()
        .map(|version| version.join("x64/rc.exe"))
        .find(|candidate| candidate.is_file())
}

fn find_on_path(tool: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    env::split_paths(&path)
        .map(|directory| directory.join(tool))
        .find(|candidate| candidate.is_file())
}
