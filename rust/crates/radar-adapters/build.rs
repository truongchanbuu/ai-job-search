fn main() {
    // Re-embed migrations when SQL changes; stable Rust does not track macro file reads.
    println!("cargo:rerun-if-changed=../../migrations");
}
