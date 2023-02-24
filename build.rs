// This helps rust compiler find SDL2 framework files
fn main() {
    println!("cargo:rustc-link-search=framework=/Library/Frameworks")
}
