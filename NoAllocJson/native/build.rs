fn main() {
    if cfg!(feature = "boehm") {
        println!(r"cargo:rustc-link-lib=mono-2.0-bdwgc");
        println!(r"cargo:rustc-link-search=lib");
    } else if cfg!(feature = "sgen") {
        println!(r"cargo:rustc-link-lib=mono-2.0-sgen");
        println!(r"cargo:rustc-link-search=C:\Program Files\Mono\lib");
    } else {
        panic!("No mono variant selected. Requires feature boehm (for bdwgc) or sgen");
    }
}
