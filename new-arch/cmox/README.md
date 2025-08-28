# CMOX - Idiomatic Rust Cryptography using STM32 CMOX

This crate provides idiomatic, type-safe Rust bindings to the STM32 CMOX (Cortex-M Optimized Crypto Stack) library. It implements standard Rust Crypto traits to ensure compatibility with the broader Rust cryptographic ecosystem.

## Features

### Hash Functions
- **SHA-1**: Secure Hash Algorithm 1
- **SHA-2**: SHA-224, SHA-256, SHA-384, SHA-512
- **SHA-3**: SHA3-224, SHA3-256, SHA3-384, SHA3-512
- **SM3**: Chinese cryptographic hash standard

### Block Ciphers
- **AES**: 128/192/256-bit keys in ECB, CBC, CFB, OFB, CTR, XTS modes
- **SM4**: Chinese block cipher standard

### AEAD (Authenticated Encryption with Associated Data)
- **AES-GCM**: AES in Galois/Counter Mode
- **AES-CCM**: AES in Counter with CBC-MAC Mode  
- **ChaCha20-Poly1305**: Modern AEAD cipher

### Message Authentication Codes (MAC)
- **HMAC**: Keyed hash-based MAC with various hash functions
- **CMAC**: Cipher-based MAC using AES
- **KMAC**: Keccak-based MAC

### Digital Signatures
- **ECDSA**: Elliptic Curve Digital Signature Algorithm (multiple curves)
- **EdDSA**: Ed25519 and Ed448 signature schemes
- **RSA**: PKCS#1 v1.5 and PSS padding
- **SM2**: Chinese digital signature standard

### Key Exchange
- **ECDH**: Elliptic Curve Diffie-Hellman (multiple curves)
- **Curve25519/Curve448**: Montgomery curves

### Random Number Generation
- **CTR-DRBG**: Counter mode deterministic random bit generator

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
cmox = { path = "../cmox", features = ["sha2", "aes", "ecdsa"] }
```

### Hash Functions

```rust
use cmox::hash::{Sha256, Sha256Hash};
use digest::{Digest, FixedOutput};

// Initialize CMOX library (must be called once)
cmox::initialize()?;

let mut hasher = Sha256::new();
hasher.update(b"hello world");
let result = hasher.finalize();

// Or use the one-shot API
let hash = Sha256::digest(b"hello world");
```

### AEAD Encryption

```rust
use cmox::aead::AesGcm128;
use aead::{Aead, KeyInit};

let key = AesGcm128::generate_key(&mut OsRng);
let cipher = AesGcm128::new(&key);
let nonce = AesGcm128::generate_nonce(&mut OsRng);

let ciphertext = cipher.encrypt(&nonce, b"plaintext message".as_ref())?;
let plaintext = cipher.decrypt(&nonce, ciphertext.as_ref())?;
```

### Digital Signatures

```rust
use cmox::signature::{EcdsaP256, EcdsaP256SigningKey};
use signature::{Signer, Verifier};

let signing_key = EcdsaP256SigningKey::random(&mut OsRng);
let verifying_key = signing_key.verifying_key();

let message = b"hello world";
let signature = signing_key.sign(message);

verifying_key.verify(message, &signature)?;
```

## Safety and Security

- All unsafe operations are encapsulated in safe wrappers
- Automatic memory management with proper cleanup
- Sensitive data is automatically zeroized when dropped
- Comprehensive error handling without information leakage
- Constant-time operations where appropriate

## no_std Support

This crate supports `no_std` environments and is suitable for embedded development:

```toml
[dependencies]
cmox = { path = "../cmox", default-features = false, features = ["sha2"] }
```

## Requirements

- STM32 CMOX library installation
- ARM GCC toolchain for cross-compilation
- Rust target: `thumbv7em-none-eabihf`

See the `cmox-sys` crate documentation for detailed setup instructions.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.