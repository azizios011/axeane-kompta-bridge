use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    // Navigate from OUT_DIR (e.g. target/debug/build/xxx/out) to target/debug or target/release
    let profile_dir = out_dir
        .parent().unwrap()  // build/xxx/
        .parent().unwrap()  // build/
        .parent().unwrap(); // debug/ or release/

    let src = manifest_dir.join("frontend");
    let dst = profile_dir.join("frontend");

    if src.exists() && !dst.exists() {
        println!("cargo:warning=Copying frontend/ to {}", dst.display());
        copy_dir_all(&src, &dst).expect("Failed to copy frontend directory");
    }
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            // Skip node_modules and .next to avoid copying huge dirs
            if entry.file_name() == "node_modules" || entry.file_name() == ".next" {
                continue;
            }
            copy_dir_all(&entry.path(), &dst_path)?;
        } else {
            fs::copy(entry.path(), &dst_path)?;
        }
    }
    Ok(())
}
