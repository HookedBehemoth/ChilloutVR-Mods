fn main() {
    let target = std::env::var("TARGET").unwrap();

    match target.as_str() {
        "x86_64-pc-windows-msvc" => {
            println!("cargo:rustc-link-arg=/NODEFAULTLIB");
        },
        "x86_64-pc-windows-gnu" => {
            println!("cargo:rustc-link-arg=-nostdlib");
        },
        _ => {}
    }
}