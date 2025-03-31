use std::process::{exit, Command};

fn main() {
    let wasm_frontend_path = "../wgpu-frontend";

    // let which_build_flag = match env::var("PROFILE").as_deref() {
    //     Ok("release") => "--release",
    //     Ok("debug") => "--dev",
    //     _ => "--dev"
    // }; // TODO: --dev hangs for some reason, but I'd like to be able to build with it
    let which_build_flag = "--release";

    let status = Command::new("wasm-pack")
        .arg("build").arg(wasm_frontend_path)
        .args(&["--out-dir", "static"])
        .arg(which_build_flag)
        .args(&["--target=web", "--no-typescript", "--no-pack"])
        .status().unwrap();
    println!("cargo::rerun-if-changed={}", wasm_frontend_path);
    println!("cargo::rerun-if-changed=build.rs");

    exit(status.code().unwrap());
}