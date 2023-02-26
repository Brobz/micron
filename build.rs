// This helps rust compiler find SDL2 framework files
fn main() {
    println!("cargo:rustc-link-search=framework=/Library/Frameworks");

    [cfg!(target_os = "macos")];
    println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path");

    [cfg!(target_os = "linux")];
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
}
