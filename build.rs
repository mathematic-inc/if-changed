//! Declares the `coverage_nightly` cfg so that `rustc` does not warn about it.

fn main() {
    println!("cargo:rustc-check-cfg=cfg(coverage_nightly)");
}
