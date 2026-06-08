use std::process::Command;

fn main() {
    let profile = std::env::var("PROFILE").unwrap_or_default();
    if profile != "release" {
        return;
    }

    println!("cargo:rerun-if-changed=frontend/app/");
    println!("cargo:rerun-if-changed=frontend/components/");
    println!("cargo:rerun-if-changed=frontend/lib/");
    println!("cargo:rerun-if-changed=frontend/package.json");
    println!("cargo:rerun-if-changed=frontend/next.config.ts");

    let frontend_dir = std::path::Path::new("frontend");

    // Install dependencies if node_modules is missing
    if !frontend_dir.join("node_modules").exists() {
        println!("cargo:warning=Installing frontend dependencies...");
        let status = Command::new("npm.cmd")
            .arg("install")
            .current_dir(frontend_dir)
            .status()
            .expect("Failed to run npm install. Is Node.js installed?");
        if !status.success() {
            panic!("npm install failed");
        }
    }

    // Build the Next.js frontend
    println!("cargo:warning=Building Next.js frontend (npm run build)...");
    let status = Command::new("npm.cmd")
        .args(["run", "build"])
        .current_dir(frontend_dir)
        .status()
        .expect("Failed to run npm run build. Is Node.js installed?");

    if !status.success() {
        panic!("Next.js frontend build failed");
    }
}
