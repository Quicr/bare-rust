use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo::rerun-if-changed=STM32F0x2.svd");

    let bindings = svdgen::Builder::default()
        .svd_file("STM32F0x2.svd")
        .include("RCC")
        .include("FLASH")
        .include("GPIOA")
        .include("GPIOB")
        .build()
        .expect("Unable to parse SVD file");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("stm32f0x2.rs"))
        .expect("Unable to translate SVD file");
}
