//! Error types for the CMOX crate

use cmox_sys::*;

/// Result type alias for CMOX operations
pub type Result<T> = core::result::Result<T, CmoxError>;

pub(crate) type CoreResult = core::result::Result<(), CoreError>;
pub(crate) type HashResult = core::result::Result<(), HashError>;
pub(crate) type CipherResult = core::result::Result<(), CipherError>;
pub(crate) type EccResult = core::result::Result<(), EccError>;
pub(crate) type RsaResult = core::result::Result<(), RsaError>;
pub(crate) type DrbgResult = core::result::Result<(), DrbgError>;
pub(crate) type MacResult = core::result::Result<(), MacError>;
pub(crate) type UtilsResult = core::result::Result<(), UtilsError>;

/// Allow for return values to be cast to typed errors
pub(crate) trait FromRetval<T> {
    /// Convert a return value into a Result, with the success return value being mapped to Ok
    fn from_rv(rv: T) -> Self;
}

/// Core library errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreError {
    /// Init operation failed
    InitFail,
}

impl defmt::Format for CoreError {
    fn format(&self, f: defmt::Formatter<'_>) {
        match self {
            CoreError::InitFail => defmt::write!(f, "InitFail"),
        }
    }
}

impl FromRetval<cmox_init_retval_t> for CoreResult {
    fn from_rv(rv: cmox_init_retval_t) -> Self {
        match rv {
            CMOX_INIT_SUCCESS => Ok(()),
            CMOX_INIT_FAIL => Err(CoreError::InitFail),
            _ => unreachable!(),
        }
    }
}

/// Hash function errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashError {
    /// Some error happens internally in the hash module
    Internal,

    /// One or more parameter has been wrongly passed to the function (e.g. pointer to NULL)
    BadParameter,

    /// Error on performing the operation (e.g. an operation has been called before initializing
    /// the handle)
    BadOperation,

    /// The desired digest size is not supported by the hash alforithm
    BadTagSize,
}

impl defmt::Format for HashError {
    fn format(&self, f: defmt::Formatter<'_>) {
        match self {
            HashError::Internal => defmt::write!(f, "Internal"),
            HashError::BadParameter => defmt::write!(f, "BadParameter"),
            HashError::BadOperation => defmt::write!(f, "BadOperation"),
            HashError::BadTagSize => defmt::write!(f, "BadTagSize"),
        }
    }
}

impl FromRetval<cmox_hash_retval_t> for HashResult {
    fn from_rv(rv: cmox_hash_retval_t) -> Self {
        match rv {
            CMOX_HASH_SUCCESS => Ok(()),
            CMOX_HASH_ERR_INTERNAL => Err(HashError::Internal),
            CMOX_HASH_ERR_BAD_PARAMETER => Err(HashError::BadParameter),
            CMOX_HASH_ERR_BAD_OPERATION => Err(HashError::BadOperation),
            CMOX_HASH_ERR_BAD_TAG_SIZE => Err(HashError::BadTagSize),
            _ => unreachable!(),
        }
    }
}

/// Cipher operation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CipherError {
    /// Some error happens internally in the cipher module
    Internal,

    /// The function is not implemented for the current algorithm
    NotImplemented,

    /// One or more parameter has been wrongly passed to the function (e.g. pointer to NULL)
    BadParameter,

    /// Error on performing the operation (e.g. an operation has been called before initializing
    /// the handle)
    BadOperation,

    /// A buffer with a wrong size has been passed to the function
    BadInputSize,

    /// Authentication of the tag failed
    AuthFail,
}

impl defmt::Format for CipherError {
    fn format(&self, f: defmt::Formatter<'_>) {
        match self {
            CipherError::Internal => defmt::write!(f, "Internal"),
            CipherError::NotImplemented => defmt::write!(f, "NotImplemented"),
            CipherError::BadParameter => defmt::write!(f, "BadParameter"),
            CipherError::BadOperation => defmt::write!(f, "BadOperation"),
            CipherError::BadInputSize => defmt::write!(f, "BadInputSize"),
            CipherError::AuthFail => defmt::write!(f, "AuthFail"),
        }
    }
}

impl FromRetval<cmox_cipher_retval_t> for CipherResult {
    fn from_rv(rv: cmox_cipher_retval_t) -> Self {
        match rv {
            CMOX_CIPHER_SUCCESS => Ok(()),
            CMOX_CIPHER_AUTH_SUCCESS => Ok(()),
            CMOX_CIPHER_ERR_INTERNAL => Err(CipherError::Internal),
            CMOX_CIPHER_ERR_NOT_IMPLEMENTED => Err(CipherError::NotImplemented),
            CMOX_CIPHER_ERR_BAD_PARAMETER => Err(CipherError::BadParameter),
            CMOX_CIPHER_ERR_BAD_OPERATION => Err(CipherError::BadOperation),
            CMOX_CIPHER_ERR_BAD_INPUT_SIZE => Err(CipherError::BadInputSize),
            CMOX_CIPHER_AUTH_FAIL => Err(CipherError::AuthFail),
            _ => unreachable!(),
        }
    }
}

/// ECC (Elliptic Curve Cryptography) errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EccError {
    /// Internal computat. error (e.g. hash)
    Internal,

    /// Bad input parameters
    BadParameter,

    /// Invalid Public Key value
    InvalidPubkey,

    /// Invalid Signature value
    InvalidSignature,

    /// Random not compliant with the API (Recall with other random material)
    WrongRandom,

    /// Not enough memory
    MemoryFail,

    /// Math customization not supported by current ECC curve
    MathCurveMismatch,

    /// ECC curve not supported by current functionality
    AlgoCurveMismatch,

    /// ECC signature not verified
    AuthFail,
}

impl defmt::Format for EccError {
    fn format(&self, f: defmt::Formatter<'_>) {
        match self {
            EccError::Internal => defmt::write!(f, "Internal"),
            EccError::BadParameter => defmt::write!(f, "BadParameter"),
            EccError::InvalidPubkey => defmt::write!(f, "InvalidPubkey"),
            EccError::InvalidSignature => defmt::write!(f, "InvalidSignature"),
            EccError::WrongRandom => defmt::write!(f, "WrongRandom"),
            EccError::MemoryFail => defmt::write!(f, "MemoryFail"),
            EccError::MathCurveMismatch => defmt::write!(f, "MathCurveMismatch"),
            EccError::AlgoCurveMismatch => defmt::write!(f, "AlgoCurveMismatch"),
            EccError::AuthFail => defmt::write!(f, "AuthFail"),
        }
    }
}

impl FromRetval<cmox_ecc_retval_t> for EccResult {
    fn from_rv(rv: cmox_ecc_retval_t) -> Self {
        match rv {
            CMOX_ECC_SUCCESS => Ok(()),
            CMOX_ECC_AUTH_SUCCESS => Ok(()),
            CMOX_ECC_ERR_INTERNAL => Err(EccError::Internal),
            CMOX_ECC_ERR_BAD_PARAMETERS => Err(EccError::BadParameter),
            CMOX_ECC_ERR_INVALID_PUBKEY => Err(EccError::InvalidPubkey),
            CMOX_ECC_ERR_INVALID_SIGNATURE => Err(EccError::InvalidSignature),
            CMOX_ECC_ERR_WRONG_RANDOM => Err(EccError::WrongRandom),
            CMOX_ECC_ERR_MEMORY_FAIL => Err(EccError::MemoryFail),
            CMOX_ECC_ERR_MATHCURVE_MISMATCH => Err(EccError::MathCurveMismatch),
            CMOX_ECC_ERR_ALGOCURVE_MISMATCH => Err(EccError::AlgoCurveMismatch),
            CMOX_ECC_AUTH_FAIL => Err(EccError::AuthFail),
            _ => unreachable!(),
        }
    }
}

/// RSA cryptography errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RsaError {
    /// Internal computat. error (e.g. hash)
    Internal,

    /// Bad input parameters
    BadParameter,

    /// Input too long for the current modulus
    ModulusTooShort,

    /// RSA invalid signature value
    InvalidSignature,

    /// RSA invalid decryption, due to mismatch between private key and input
    WrongDecryption,

    /// Random not compliant with the API (Recall with other random material)
    WrongRandom,

    /// Not enough memory
    MemoryFail,

    /// Math customization not supported by current functionality
    MathAlgoMismatch,

    /// Modexp function not supported by current functionality
    MexpAlgoMismatch,

    /// ECC signature not verified
    AuthFail,
}

impl defmt::Format for RsaError {
    fn format(&self, f: defmt::Formatter<'_>) {
        match self {
            RsaError::Internal => defmt::write!(f, "Internal"),
            RsaError::BadParameter => defmt::write!(f, "BadParameter"),
            RsaError::ModulusTooShort => defmt::write!(f, "ModulusTooShort"),
            RsaError::InvalidSignature => defmt::write!(f, "InvalidSignature"),
            RsaError::WrongDecryption => defmt::write!(f, "WrongDecryption"),
            RsaError::WrongRandom => defmt::write!(f, "WrongRandom"),
            RsaError::MemoryFail => defmt::write!(f, "MemoryFail"),
            RsaError::MathAlgoMismatch => defmt::write!(f, "MathAlgoMismatch"),
            RsaError::MexpAlgoMismatch => defmt::write!(f, "MexpAlgoMismatch"),
            RsaError::AuthFail => defmt::write!(f, "AuthFail"),
        }
    }
}

impl FromRetval<cmox_rsa_retval_t> for RsaResult {
    fn from_rv(rv: cmox_rsa_retval_t) -> Self {
        match rv {
            CMOX_RSA_SUCCESS => Ok(()),
            CMOX_RSA_AUTH_SUCCESS => Ok(()),
            CMOX_RSA_ERR_INTERNAL => Err(RsaError::Internal),
            CMOX_RSA_ERR_BAD_PARAMETER => Err(RsaError::BadParameter),
            CMOX_RSA_ERR_MODULUS_TOO_SHORT => Err(RsaError::ModulusTooShort),
            CMOX_RSA_ERR_INVALID_SIGNATURE => Err(RsaError::InvalidSignature),
            CMOX_RSA_ERR_WRONG_DECRYPTION => Err(RsaError::WrongRandom),
            CMOX_RSA_ERR_WRONG_RANDOM => Err(RsaError::WrongRandom),
            CMOX_RSA_ERR_MEMORY_FAIL => Err(RsaError::MemoryFail),
            CMOX_RSA_ERR_MATH_ALGO_MISMATCH => Err(RsaError::MathAlgoMismatch),
            CMOX_RSA_ERR_MEXP_ALGO_MISMATCH => Err(RsaError::MexpAlgoMismatch),
            CMOX_RSA_AUTH_FAIL => Err(RsaError::AuthFail),
            _ => unreachable!(),
        }
    }
}

/// DRBG (Deterministic Random Bit Generator) errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrbgError {
    /// Generic internal error
    Internal,

    /// One of the expected function parameters is invalid
    BadParameter,

    /// Invalid operation
    BadOperation,

    /// DRBG has not been correctly initialized
    UninitializedState,

    /// Reseed is needed
    ReseedNeeded,

    /// Check the size of the entropy string
    BadEntropySize,

    /// Check the size of the personalization string
    BadPersonalizationStringLength,

    /// Check the size of the additional input string
    BadAdditionalInputLength,

    /// Check the size of the random request
    BadRequest,

    /// Check the size of the nonce
    BadNonceSize,
}

impl defmt::Format for DrbgError {
    fn format(&self, f: defmt::Formatter<'_>) {
        match self {
            DrbgError::Internal => defmt::write!(f, "Internal"),
            DrbgError::BadParameter => defmt::write!(f, "BadParameter"),
            DrbgError::BadOperation => defmt::write!(f, "BadOperation"),
            DrbgError::UninitializedState => defmt::write!(f, "UninitializedState"),
            DrbgError::ReseedNeeded => defmt::write!(f, "ReseedNeeded"),
            DrbgError::BadEntropySize => defmt::write!(f, "BadEntropySize"),
            DrbgError::BadPersonalizationStringLength => defmt::write!(f, "BadPersonalizationStringLength"),
            DrbgError::BadAdditionalInputLength => defmt::write!(f, "BadAdditionalInputLength"),
            DrbgError::BadRequest => defmt::write!(f, "BadRequest"),
            DrbgError::BadNonceSize => defmt::write!(f, "BadNonceSize"),
        }
    }
}

impl FromRetval<cmox_drbg_retval_t> for DrbgResult {
    fn from_rv(rv: cmox_drbg_retval_t) -> Self {
        match rv {
            CMOX_DRBG_SUCCESS => Ok(()),
            CMOX_DRBG_ERR_INTERNAL => Err(DrbgError::Internal),
            CMOX_DRBG_ERR_BAD_PARAMETER => Err(DrbgError::BadParameter),
            CMOX_DRBG_ERR_BAD_OPERATION => Err(DrbgError::BadOperation),
            CMOX_DRBG_ERR_UNINIT_STATE => Err(DrbgError::UninitializedState),
            CMOX_DRBG_ERR_RESEED_NEEDED => Err(DrbgError::ReseedNeeded),
            CMOX_DRBG_ERR_BAD_ENTROPY_SIZE => Err(DrbgError::BadEntropySize),
            CMOX_DRBG_ERR_BAD_PERS_STR_LEN => Err(DrbgError::BadPersonalizationStringLength),
            CMOX_DRBG_ERR_BAD_ADD_INPUT_LEN => Err(DrbgError::BadAdditionalInputLength),
            CMOX_DRBG_ERR_BAD_REQUEST => Err(DrbgError::BadRequest),
            CMOX_DRBG_ERR_BAD_NONCE_SIZE => Err(DrbgError::BadNonceSize),
            _ => unreachable!(),
        }
    }
}

/// MAC (Message Authentication Code) errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacError {
    /// Some error happens internally in the MAC module
    Internal,

    /// One or more parameter has been wrongly passed to the function (e.g. pointer to NULL)
    BadParameter,

    /// Error on performing the operation (e.g. an operation has been called before initializing
    /// the handle)
    BadOperation,

    /// Authentication of the tag failed
    AuthFail,
}

impl defmt::Format for MacError {
    fn format(&self, f: defmt::Formatter<'_>) {
        match self {
            MacError::Internal => defmt::write!(f, "Internal"),
            MacError::BadParameter => defmt::write!(f, "BadParameter"),
            MacError::BadOperation => defmt::write!(f, "BadOperation"),
            MacError::AuthFail => defmt::write!(f, "AuthFail"),
        }
    }
}

impl FromRetval<cmox_mac_retval_t> for MacResult {
    fn from_rv(rv: cmox_mac_retval_t) -> Self {
        match rv {
            CMOX_MAC_SUCCESS => Ok(()),
            CMOX_MAC_ERR_INTERNAL => Err(MacError::Internal),
            CMOX_MAC_ERR_BAD_PARAMETER => Err(MacError::BadParameter),
            CMOX_MAC_ERR_BAD_OPERATION => Err(MacError::BadOperation),
            CMOX_MAC_AUTH_FAIL => Err(MacError::AuthFail),
            _ => unreachable!(),
        }
    }
}

/// Utils errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UtilsError {
    /// Input buffers are different
    AuthFail,
}

impl defmt::Format for UtilsError {
    fn format(&self, f: defmt::Formatter<'_>) {
        match self {
            UtilsError::AuthFail => defmt::write!(f, "AuthFail"),
        }
    }
}

impl FromRetval<cmox_utils_retval_t> for UtilsResult {
    fn from_rv(rv: cmox_utils_retval_t) -> Self {
        match rv {
            CMOX_UTILS_AUTH_SUCCESS => Ok(()),
            CMOX_UTILS_AUTH_FAIL => Err(UtilsError::AuthFail),
            _ => unreachable!(),
        }
    }
}

/// Top-level hierarchical error enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmoxError {
    /// Core library errors (initialization, etc.)
    Core(CoreError),

    /// Hash function errors
    Hash(HashError),

    /// Cipher operation errors
    Cipher(CipherError),

    /// ECC (ECDSA, EdDSA, ECDH) errors
    Ecc(EccError),

    /// RSA cryptography errors
    Rsa(RsaError),

    /// DRBG (random number generation) errors
    Drbg(DrbgError),

    /// MAC (message authentication code) errors
    Mac(MacError),

    /// Utils errors
    Utils(UtilsError),
}

impl From<CoreError> for CmoxError {
    fn from(err: CoreError) -> Self {
        Self::Core(err)
    }
}

impl From<HashError> for CmoxError {
    fn from(err: HashError) -> Self {
        Self::Hash(err)
    }
}

impl From<CipherError> for CmoxError {
    fn from(err: CipherError) -> Self {
        Self::Cipher(err)
    }
}

impl From<EccError> for CmoxError {
    fn from(err: EccError) -> Self {
        Self::Ecc(err)
    }
}

impl From<RsaError> for CmoxError {
    fn from(err: RsaError) -> Self {
        Self::Rsa(err)
    }
}

impl From<DrbgError> for CmoxError {
    fn from(err: DrbgError) -> Self {
        Self::Drbg(err)
    }
}

impl From<MacError> for CmoxError {
    fn from(err: MacError) -> Self {
        Self::Mac(err)
    }
}

impl From<UtilsError> for CmoxError {
    fn from(err: UtilsError) -> Self {
        Self::Utils(err)
    }
}

impl defmt::Format for CmoxError {
    fn format(&self, f: defmt::Formatter<'_>) {
        match self {
            CmoxError::Core(err) => defmt::write!(f, "Core error: {:?}", err),
            CmoxError::Hash(err) => defmt::write!(f, "Hash error: {:?}", err),
            CmoxError::Cipher(err) => defmt::write!(f, "Cipher error: {:?}", err),
            CmoxError::Ecc(err) => defmt::write!(f, "ECC error: {:?}", err),
            CmoxError::Rsa(err) => defmt::write!(f, "RSA error: {:?}", err),
            CmoxError::Drbg(err) => defmt::write!(f, "DRBG error: {:?}", err),
            CmoxError::Mac(err) => defmt::write!(f, "MAC error: {:?}", err),
            CmoxError::Utils(err) => defmt::write!(f, "Utils error: {:?}", err),
        }
    }
}
