use std::env;
use std::path::PathBuf;

fn main() {
    // Get the project root directory
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let lib_path = PathBuf::from(&manifest_dir).join("lib");

    // Tell cargo to look for libraries in the lib directory
    println!("cargo:rustc-link-search=native={}", lib_path.display());
    
    // On Linux, also set the rpath so the binary can find the library at runtime
    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../lib");
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_path.display());
    }
    
    // Rerun if the lib directory changes
    println!("cargo:rerun-if-changed=lib/");
}

