//! Error types for the CMOX crate

use core::fmt;

/// Result type alias for CMOX operations
pub type Result<T> = core::result::Result<T, CmoxError>;

/// Errors that can occur when using CMOX cryptographic operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmoxError {
    /// CMOX library initialization failed
    InitializationFailed,
    /// CMOX library finalization failed
    FinalizationFailed,
    /// CMOX library is not initialized
    NotInitialized,
    /// Invalid parameter provided to CMOX function
    InvalidParameter,
    /// Bad operation or state
    BadOperation,
    /// Internal error in CMOX library
    InternalError,
    /// Memory allocation failure
    MemoryFailure,
    /// Invalid input size
    InvalidInputSize,
    /// Invalid input data
    InvalidInput,
    /// Authentication/verification failed
    AuthenticationFailed,
    /// Invalid signature
    InvalidSignature,
    /// Invalid public key
    InvalidPublicKey,
    /// Wrong random number
    WrongRandom,
    /// Algorithm/curve mismatch
    AlgorithmMismatch,
    /// Modulus too short (RSA)
    ModulusTooShort,
    /// Wrong decryption (RSA)
    WrongDecryption,
    /// Uninitialized state (DRBG)
    UninitializedState,
    /// Reseed needed (DRBG)
    ReseedNeeded,
    /// Invalid entropy size (DRBG)
    InvalidEntropySize,
    /// Invalid personalization string length (DRBG)
    InvalidPersonalizationLength,
    /// Invalid additional input length (DRBG)
    InvalidAdditionalInputLength,
    /// Invalid request (DRBG)
    InvalidRequest,
    /// Invalid nonce size (DRBG)
    InvalidNonceSize,
    /// Invalid tag size
    InvalidTagSize,
}

impl CmoxError {
    /// Convert CMOX hash return code to error
    pub(crate) fn from_hash_retval(retval: u32) -> Result<()> {
        match retval {
            0x00020000 => Ok(()), // CMOX_HASH_SUCCESS
            0x00020003 => Err(CmoxError::InvalidParameter), // CMOX_HASH_ERR_BAD_PARAMETER
            0x00020004 => Err(CmoxError::BadOperation), // CMOX_HASH_ERR_BAD_OPERATION
            0x00020006 => Err(CmoxError::InvalidTagSize), // CMOX_HASH_ERR_BAD_TAG_SIZE
            0x00020001 => Err(CmoxError::InternalError), // CMOX_HASH_ERR_INTERNAL
            _ => Err(CmoxError::InternalError),
        }
    }

    /// Convert CMOX cipher return code to error
    pub(crate) fn from_cipher_retval(retval: u32) -> Result<()> {
        match retval {
            0x00010000 => Ok(()), // CMOX_CIPHER_SUCCESS
            0x00010003 => Err(CmoxError::InvalidParameter), // CMOX_CIPHER_ERR_BAD_PARAMETER
            0x00010004 => Err(CmoxError::BadOperation), // CMOX_CIPHER_ERR_BAD_OPERATION
            0x00010005 => Err(CmoxError::InvalidInputSize), // CMOX_CIPHER_ERR_BAD_INPUT_SIZE
            0x00010001 => Err(CmoxError::InternalError), // CMOX_CIPHER_ERR_INTERNAL
            0x00010002 => Err(CmoxError::InternalError), // CMOX_CIPHER_ERR_NOT_IMPLEMENTED
            0x00016E93 => Err(CmoxError::AuthenticationFailed), // CMOX_CIPHER_AUTH_FAIL
            _ => Err(CmoxError::InternalError),
        }
    }

    /// Convert CMOX ECC return value to Result
    pub(crate) fn from_ecc_retval(retval: u32) -> Result<()> {
        match retval {
            0x00000000 => Ok(()), // CMOX_ECC_SUCCESS
            0x00050001 => Err(CmoxError::InvalidParameter), // CMOX_ECC_ERR_BAD_PARAMETERS
            0x00050002 => Err(CmoxError::InternalError), // CMOX_ECC_ERR_MATHCURVE_MISMATCH
            0x00050003 => Err(CmoxError::InternalError), // CMOX_ECC_ERR_ALGOCURVE_MISMATCH
            0x00050004 => Err(CmoxError::InternalError), // CMOX_ECC_ERR_MEMORY_FAIL
            0x00050005 => Err(CmoxError::InvalidInput), // CMOX_ECC_ERR_WRONG_RANDOM
            0x00050006 => Err(CmoxError::AuthenticationFailed), // CMOX_ECC_AUTH_FAIL
            _ => Err(CmoxError::InternalError),
        }
    }

    /// Convert CMOX RSA return value to Result
    pub(crate) fn from_rsa_retval(retval: u32) -> Result<()> {
        match retval {
            0x00050000 => Ok(()), // CMOX_RSA_SUCCESS
            0x0005C726 => Ok(()), // CMOX_RSA_AUTH_SUCCESS (signature verification success)
            0x00050003 => Err(CmoxError::InvalidParameter), // CMOX_RSA_ERR_BAD_PARAMETER
            0x00050001 => Err(CmoxError::InternalError), // CMOX_RSA_ERR_INTERNAL
            0x00050007 => Err(CmoxError::ModulusTooShort), // CMOX_RSA_ERR_MODULUS_TOO_SHORT
            0x00050009 => Err(CmoxError::InvalidSignature), // CMOX_RSA_ERR_INVALID_SIGNATURE
            0x0005000A => Err(CmoxError::WrongDecryption), // CMOX_RSA_ERR_WRONG_DECRYPTION
            0x0005000B => Err(CmoxError::WrongRandom), // CMOX_RSA_ERR_WRONG_RANDOM
            0x0005000C => Err(CmoxError::MemoryFailure), // CMOX_RSA_ERR_MEMORY_FAIL
            0x00050010 => Err(CmoxError::AlgorithmMismatch), // CMOX_RSA_ERR_MATH_ALGO_MISMATCH
            0x00050011 => Err(CmoxError::AlgorithmMismatch), // CMOX_RSA_ERR_MEXP_ALGO_MISMATCH
            0x00056E93 => Err(CmoxError::AuthenticationFailed), // CMOX_RSA_AUTH_FAIL
            _ => Err(CmoxError::InternalError),
        }
    }

    /// Convert CMOX DRBG return value to Result
    pub(crate) fn from_drbg_retval(retval: u32) -> Result<()> {
        match retval {
            0x00040000 => Ok(()), // CMOX_DRBG_SUCCESS
            0x00040003 => Err(CmoxError::InvalidParameter), // CMOX_DRBG_ERR_BAD_PARAMETER
            0x00040001 => Err(CmoxError::InternalError), // CMOX_DRBG_ERR_INTERNAL
            0x00040004 => Err(CmoxError::BadOperation), // CMOX_DRBG_ERR_BAD_OPERATION
            0x0004000D => Err(CmoxError::UninitializedState), // CMOX_DRBG_ERR_UNINIT_STATE
            0x0004000E => Err(CmoxError::ReseedNeeded), // CMOX_DRBG_ERR_RESEED_NEEDED
            0x0004000F => Err(CmoxError::InvalidEntropySize), // CMOX_DRBG_ERR_BAD_ENTROPY_SIZE
            0x00040010 => Err(CmoxError::InvalidPersonalizationLength), // CMOX_DRBG_ERR_BAD_PERS_STR_LEN
            0x00040011 => Err(CmoxError::InvalidAdditionalInputLength), // CMOX_DRBG_ERR_BAD_ADD_INPUT_LEN
            0x00040012 => Err(CmoxError::InvalidRequest), // CMOX_DRBG_ERR_BAD_REQUEST
            0x00040013 => Err(CmoxError::InvalidNonceSize), // CMOX_DRBG_ERR_BAD_NONCE_SIZE
            _ => Err(CmoxError::InternalError),
        }
    }
}

impl fmt::Display for CmoxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InitializationFailed => write!(f, "CMOX library initialization failed"),
            Self::FinalizationFailed => write!(f, "CMOX library finalization failed"),
            Self::NotInitialized => write!(f, "CMOX library is not initialized"),
            Self::InvalidParameter => write!(f, "Invalid parameter provided"),
            Self::BadOperation => write!(f, "Bad operation or state"),
            Self::InternalError => write!(f, "Internal error in CMOX library"),
            Self::MemoryFailure => write!(f, "Memory allocation failure"),
            Self::InvalidInputSize => write!(f, "Invalid input size"),
            Self::InvalidInput => write!(f, "Invalid input data"),
            Self::AuthenticationFailed => write!(f, "Authentication or verification failed"),
            Self::InvalidSignature => write!(f, "Invalid signature"),
            Self::InvalidPublicKey => write!(f, "Invalid public key"),
            Self::WrongRandom => write!(f, "Wrong random number"),
            Self::AlgorithmMismatch => write!(f, "Algorithm or curve mismatch"),
            Self::ModulusTooShort => write!(f, "RSA modulus too short"),
            Self::WrongDecryption => write!(f, "Wrong decryption"),
            Self::UninitializedState => write!(f, "Uninitialized state"),
            Self::ReseedNeeded => write!(f, "Reseed needed"),
            Self::InvalidEntropySize => write!(f, "Invalid entropy size"),
            Self::InvalidPersonalizationLength => write!(f, "Invalid personalization string length"),
            Self::InvalidAdditionalInputLength => write!(f, "Invalid additional input length"),
            Self::InvalidRequest => write!(f, "Invalid request"),
            Self::InvalidNonceSize => write!(f, "Invalid nonce size"),
            Self::InvalidTagSize => write!(f, "Invalid tag size"),
        }
    }
}

