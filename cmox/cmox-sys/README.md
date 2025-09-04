# cmox-sys

Rust bindings to the STM32 CMOX (Cortex-M Optimized Crypto Stack) library.

## Features

This crate provides low-level bindings to the STM32 CMOX cryptographic library and supports different Cortex-M targets through cargo features:

- `cm0` - Cortex-M0 target
- `cm0plus` - Cortex-M0+ target  
- `cm3` - Cortex-M3 target
- `cm4` - Cortex-M4 target (default)
- `cm7` - Cortex-M7 target
- `cm33` - Cortex-M33 target

## Requirements

### Environment Variables

Before building, you must set the following environment variables:

- **`CMOX_PATH`**: Path to the STM32 CMOX library root directory
  - This directory should contain `include/` and `lib/` subdirectories

- **`ARM_STDLIB_PATH`**: Path to ARM GCC standard library headers  
  - Required for cross-compilation to ARM targets

### Build Dependencies

- ARM GCC toolchain for cross-compilation
- `clang` for bindgen
- Rust target: `thumbv7em-none-eabihf` (install with `rustup target add thumbv7em-none-eabihf`)

## Building

Set the environment variables and build for the ARM target:

```bash
export ARM_STDLIB_PATH="/Applications/ArmGNUToolchain/14.2.rel1/arm-none-eabi/arm-none-eabi/include/"
export CMOX_PATH="/path/to/STM32CubeExpansion_Crypto_V4.1.0/Middlewares/ST/STM32_Cryptographic"

cargo build --target=thumbv7em-none-eabihf
```

Or use them inline:

```bash
ARM_STDLIB_PATH=/Applications/ArmGNUToolchain/14.2.rel1/arm-none-eabi/arm-none-eabi/include/ \
CMOX_PATH=/path/to/STM32CubeExpansion_Crypto_V4.1.0/Middlewares/ST/STM32_Cryptographic \
cargo build --target=thumbv7em-none-eabihf
```

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
cmox-sys = { path = "../cmox-sys", features = ["cm4"] }
```

The crate exposes all functions and constants from the CMOX library headers:

```rust
use cmox_sys::*;

// Initialize the library
let init_arg = cmox_init_arg_t {
    target: CMOX_INIT_TARGET_AUTO,
    pArg: core::ptr::null_mut(),
};
let result = unsafe { cmox_initialize(&init_arg as *const _ as *mut _) };
```

## Library Structure

The CMOX library provides the following cryptographic modules:

- **Hash**: SHA-1, SHA-2 (224/256/384/512), SHA-3, SM3
- **Cipher**: AES, ChaCha20-Poly1305, various modes (ECB, CBC, CTR, GCM, etc.)
- **MAC**: HMAC, CMAC, KMAC
- **RSA**: PKCS#1 v1.5 and v2.2 (PSS/OAEP)
- **ECC**: ECDSA, ECDH, EdDSA (Ed25519/Ed448), multiple curves
- **DRBG**: CTR-DRBG for random number generation
- **Utils**: Constant-time comparison utilities

## Safety

All functions in this crate are marked as `unsafe` as they are direct bindings to C functions. Users must ensure proper initialization, memory management, and parameter validation when using these functions.
