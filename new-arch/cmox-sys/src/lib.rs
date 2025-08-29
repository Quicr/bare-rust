#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Manual definitions for #define macros that bindgen doesn't capture
// (Only the ones with type casts that bindgen can't handle automatically)
pub const CMOX_INIT_SUCCESS: cmox_init_retval_t = 0x00000000;
pub const CMOX_INIT_FAIL: cmox_init_retval_t = 0x00000001;

pub const CMOX_INIT_TARGET_AUTO: cmox_init_target_t = 0x00000000;
pub const CMOX_INIT_TARGET_F0: cmox_init_target_t = 0x46300000;
pub const CMOX_INIT_TARGET_F1: cmox_init_target_t = 0x46310000;
pub const CMOX_INIT_TARGET_F2: cmox_init_target_t = 0x46320000;
pub const CMOX_INIT_TARGET_F3: cmox_init_target_t = 0x46330000;
pub const CMOX_INIT_TARGET_F4: cmox_init_target_t = 0x46340000;
pub const CMOX_INIT_TARGET_F7: cmox_init_target_t = 0x46370000;
pub const CMOX_INIT_TARGET_H5: cmox_init_target_t = 0x48350000;
pub const CMOX_INIT_TARGET_H7: cmox_init_target_t = 0x48370000;
pub const CMOX_INIT_TARGET_H7AB: cmox_init_target_t = 0x48378000;
pub const CMOX_INIT_TARGET_G0: cmox_init_target_t = 0x47300000;
pub const CMOX_INIT_TARGET_G4: cmox_init_target_t = 0x47340000;
pub const CMOX_INIT_TARGET_L0: cmox_init_target_t = 0x4C300000;
pub const CMOX_INIT_TARGET_L1: cmox_init_target_t = 0x4C310000;
pub const CMOX_INIT_TARGET_L4: cmox_init_target_t = 0x4C340000;
pub const CMOX_INIT_TARGET_L5: cmox_init_target_t = 0x4C350000;
pub const CMOX_INIT_TARGET_WB: cmox_init_target_t = 0x57420000;
pub const CMOX_INIT_TARGET_WBA: cmox_init_target_t = 0x57424100;
pub const CMOX_INIT_TARGET_WL: cmox_init_target_t = 0x574C0000;

pub const CMOX_HASH_SUCCESS: cmox_hash_retval_t = 0x00020000;
pub const CMOX_HASH_ERR_INTERNAL: cmox_hash_retval_t = 0x00020001;
pub const CMOX_HASH_ERR_BAD_PARAMETER: cmox_hash_retval_t = 0x00020003;
pub const CMOX_HASH_ERR_BAD_OPERATION: cmox_hash_retval_t = 0x00020004;
pub const CMOX_HASH_ERR_BAD_TAG_SIZE: cmox_hash_retval_t = 0x00020006;

pub const CMOX_CIPHER_SUCCESS: cmox_cipher_retval_t = 0x00010000;
pub const CMOX_CIPHER_ERR_INTERNAL: cmox_cipher_retval_t = 0x00010001;
pub const CMOX_CIPHER_ERR_NOT_IMPLEMENTED: cmox_cipher_retval_t = 0x00010002;
pub const CMOX_CIPHER_ERR_BAD_PARAMETER: cmox_cipher_retval_t = 0x00010003;
pub const CMOX_CIPHER_ERR_BAD_OPERATION: cmox_cipher_retval_t = 0x00010004;
pub const CMOX_CIPHER_ERR_BAD_INPUT_SIZE: cmox_cipher_retval_t = 0x00010005;
pub const CMOX_CIPHER_AUTH_SUCCESS: cmox_cipher_retval_t = 0x0001C726;
pub const CMOX_CIPHER_AUTH_FAIL: cmox_cipher_retval_t = 0x00016E93;

pub const CMOX_ECC_SUCCESS: cmox_ecc_retval_t = 0x00060000;
pub const CMOX_ECC_ERR_INTERNAL: cmox_ecc_retval_t = 0x00060001;
pub const CMOX_ECC_ERR_BAD_PARAMETERS: cmox_ecc_retval_t = 0x00060003;
pub const CMOX_ECC_ERR_INVALID_PUBKEY: cmox_ecc_retval_t = 0x00060008;
pub const CMOX_ECC_ERR_INVALID_SIGNATURE: cmox_ecc_retval_t = 0x00060009;
pub const CMOX_ECC_ERR_WRONG_RANDOM: cmox_ecc_retval_t = 0x0006000B;
pub const CMOX_ECC_ERR_MEMORY_FAIL: cmox_ecc_retval_t = 0x0006000C;
pub const CMOX_ECC_ERR_MATHCURVE_MISMATCH: cmox_ecc_retval_t = 0x0006000E;
pub const CMOX_ECC_ERR_ALGOCURVE_MISMATCH: cmox_ecc_retval_t = 0x0006000F;
pub const CMOX_ECC_AUTH_SUCCESS: cmox_ecc_retval_t = 0x0006C726;
pub const CMOX_ECC_AUTH_FAIL: cmox_ecc_retval_t = 0x00066E93;

pub const CMOX_RSA_SUCCESS: cmox_rsa_retval_t = 0x00050000;
pub const CMOX_RSA_ERR_INTERNAL: cmox_rsa_retval_t = 0x00050001;
pub const CMOX_RSA_ERR_BAD_PARAMETER: cmox_rsa_retval_t = 0x00050003;
pub const CMOX_RSA_ERR_MODULUS_TOO_SHORT: cmox_rsa_retval_t = 0x00050007;
pub const CMOX_RSA_ERR_INVALID_SIGNATURE: cmox_rsa_retval_t = 0x00050009;
pub const CMOX_RSA_ERR_WRONG_DECRYPTION: cmox_rsa_retval_t = 0x0005000A;
pub const CMOX_RSA_ERR_WRONG_RANDOM: cmox_rsa_retval_t = 0x0005000B;
pub const CMOX_RSA_ERR_MEMORY_FAIL: cmox_rsa_retval_t = 0x0005000C;
pub const CMOX_RSA_ERR_MATH_ALGO_MISMATCH: cmox_rsa_retval_t = 0x00050010;
pub const CMOX_RSA_ERR_MEXP_ALGO_MISMATCH: cmox_rsa_retval_t = 0x00050011;
pub const CMOX_RSA_AUTH_SUCCESS: cmox_rsa_retval_t = 0x0005C726;
pub const CMOX_RSA_AUTH_FAIL: cmox_rsa_retval_t = 0x00056E93;

pub const CMOX_DRBG_SUCCESS: cmox_drbg_retval_t = 0x00040000;
pub const CMOX_DRBG_ERR_INTERNAL: cmox_drbg_retval_t = 0x00040001;
pub const CMOX_DRBG_ERR_BAD_PARAMETER: cmox_drbg_retval_t = 0x00040003;
pub const CMOX_DRBG_ERR_BAD_OPERATION: cmox_drbg_retval_t = 0x00040004;
pub const CMOX_DRBG_ERR_UNINIT_STATE: cmox_drbg_retval_t = 0x0004000D;
pub const CMOX_DRBG_ERR_RESEED_NEEDED: cmox_drbg_retval_t = 0x0004000E;
pub const CMOX_DRBG_ERR_BAD_ENTROPY_SIZE: cmox_drbg_retval_t = 0x0004000F;
pub const CMOX_DRBG_ERR_BAD_PERS_STR_LEN: cmox_drbg_retval_t = 0x00040010;
pub const CMOX_DRBG_ERR_BAD_ADD_INPUT_LEN: cmox_drbg_retval_t = 0x00040011;
pub const CMOX_DRBG_ERR_BAD_REQUEST: cmox_drbg_retval_t = 0x00040012;
pub const CMOX_DRBG_ERR_BAD_NONCE_SIZE: cmox_drbg_retval_t = 0x00040013;

pub const CMOX_MAC_SUCCESS: cmox_mac_retval_t = 0x00030000;
pub const CMOX_MAC_ERR_INTERNAL: cmox_mac_retval_t = 0x00030001;
pub const CMOX_MAC_ERR_BAD_PARAMETER: cmox_mac_retval_t = 0x00030002;
pub const CMOX_MAC_ERR_BAD_OPERATION: cmox_mac_retval_t = 0x00030003;
pub const CMOX_MAC_AUTH_SUCCESS: cmox_mac_retval_t = 0x0003C726;
pub const CMOX_MAC_AUTH_FAIL: cmox_mac_retval_t = 0x00036E93;

pub const CMOX_UTILS_AUTH_SUCCESS: cmox_utils_retval_t = 0x0007C726;
pub const CMOX_UTILS_AUTH_FAIL: cmox_utils_retval_t = 0x00076E93;
