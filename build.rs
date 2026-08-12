use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap();
    
    if target.contains("android") {
        println!("cargo:rustc-link-lib=android");
        println!("cargo:rustc-link-lib=log");
        
        let android_home = env::var("ANDROID_HOME").unwrap_or_default();
        let ndk_home = env::var("ANDROID_NDK").unwrap_or_default();
        
        println!("cargo:rerun-if-env-changed=ANDROID_HOME");
        println!("cargo:rerun-if-env-changed=ANDROID_NDK");
    }
}
