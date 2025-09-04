# svd

This crate contains tooling to convert an SVD file in to Rust code that provides
reasonably safe access to device registers.  The tool reads an SVD file and
outputs a Rust file with the following contents:

* Definitions for a `Field` struct and marker structs for access control levels
* For each `<peripheral>`, a Rust module of the same name
* For each `<register>`, a submodule of its peripheral's module
* For each `<field>`, a specialization of the `Field` struct

For example, if a device has 

The output file is self-contained, with no dependencies, and is suitable for inclusion
as a module in another project.

## build.rs

Usage is similar to
[`bindgen`](https://rust-lang.github.io/rust-bindgen/tutorial-3.html).  In
`build.rs`:

``` rust
use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo::rerun-if-changed=DEVICE.svd");

    let bindings = svdgen::Builder::default()
        .svd_file("DEVICE.svd")
        .include("GPIOA")
        .include("UART1")
        .build()
        .expect("Unable to parse SVD file");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("DEVICE.rs"))
        .expect("Unable to translate SVD file");
}
```

In a file within the crate:

``` rust
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(clippy::identity_op)]
#![allow(unused_imports)]

include!(concat!(env!("OUT_DIR"), "/DEVICE.rs"));
```

## Command line

```
> cargo run -- --help
Usage: svdgen [OPTIONS] <SVD_FILE> <RUST_FILE>

Arguments:
  <SVD_FILE>   The path of the SVD file to parse
  <RUST_FILE>  The path of the Rust file to output

Options:
      --only <ONLY>  If this vector is non-empty, then only the listed peripherals are included
  -h, --help         Print help
```


