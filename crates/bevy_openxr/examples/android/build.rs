use std::{env, ffi::OsString, fs, io::Read, path::Path};

pub const OPENXR_VERSION: &str = "1.1.38";

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    if env::var_os("CARGO_CFG_TARGET_OS") != Some(OsString::from("android")) {
        return;
    }
    let mut resp = reqwest::blocking::get(
        format!(
    "https://github.com/KhronosGroup/OpenXR-SDK-Source/releases/download/release-{}/openxr_loader_for_android-{}.aar",
    OPENXR_VERSION,OPENXR_VERSION)).unwrap();
    if !resp.status().is_success() {
        eprintln!("ERROR: Unable to get OpenXR loader from github release");
        return;
    }
    let mut file = Vec::new();
    resp.read_to_end(&mut file).unwrap();
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("openxr_loader_android.arr");
    fs::write(&dest_path, file).unwrap();
    let file = fs::File::open(&dest_path).unwrap();

    let mut zip_file = zip::ZipArchive::new(file).unwrap();
    eprintln!("{:#?}", zip_file.file_names().collect::<Vec<_>>());
    let mut loader_file = zip_file
        .by_name("prefab/modules/openxr_loader/libs/android.arm64-v8a/libopenxr_loader.so")
        .unwrap();
    let mut file = Vec::new();
    loader_file.read_to_end(&mut file).unwrap();
    let out_dir = env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("./runtime_libs/arm64-v8a/libopenxr_loader.so");
    let _ = fs::create_dir_all(dest_path.parent().unwrap());
    fs::write(&dest_path, file).unwrap();
}
