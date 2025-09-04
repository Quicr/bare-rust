use std::env;
use std::path::PathBuf;

fn main() {
    // Locate the ARM stdlib and the CMOX library
    let arm_stdlib_path =
        PathBuf::from(env::var("ARM_STDLIB_PATH").expect("Please set ARM_STDLIB_PATH"));
    let cmox_path = PathBuf::from(env::var("CMOX_PATH").expect("Please set CMOX_PATH"));
    let cmox_include_path = cmox_path.join("include");
    let cmox_lib_path = cmox_path.join("lib");

    println!("cargo:rerun-if-changed={}", cmox_include_path.display());
    println!("cargo:rustc-link-search=native={}", cmox_lib_path.display());
    println!("cargo:include={}", cmox_include_path.display());

    // Select library based on target feature
    let lib_name =
        if env::var("CARGO_FEATURE_CM0").is_ok() || env::var("CARGO_FEATURE_CM0PLUS").is_ok() {
            "STM32Cryptographic_CM0_CM0PLUS"
        } else if env::var("CARGO_FEATURE_CM3").is_ok() {
            "STM32Cryptographic_CM3"
        } else if env::var("CARGO_FEATURE_CM4").is_ok() {
            "STM32Cryptographic_CM4"
        } else if env::var("CARGO_FEATURE_CM7").is_ok() {
            "STM32Cryptographic_CM7"
        } else if env::var("CARGO_FEATURE_CM33").is_ok() {
            "STM32Cryptographic_CM33"
        } else {
            "STM32Cryptographic_CM4" // default
        };

    println!("cargo:rustc-link-lib=static={}", lib_name);

    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", arm_stdlib_path.display()))
        .clang_arg(format!("-I{}", cmox_include_path.display()))
        .clang_arg("-target")
        .clang_arg("arm-none-eabi")
        .use_core()
        .derive_debug(true)
        .derive_default(true)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
