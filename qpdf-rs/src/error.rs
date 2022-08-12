use std::ffi::NulError;
use std::fmt;

use crate::Result;

/// Error codes returned by QPDF library calls
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Hash)]
pub enum QPdfErrorCode {
    Unknown,
    InvalidParameter,
    InternalError,
    SystemError,
    Unsupported,
    InvalidPassword,
    DamagedPdf,
    PagesError,
    ObjectError,
}

pub(crate) fn error_or_ok(error: qpdf_sys::qpdf_error_code_e) -> Result<()> {
    let code = match error as qpdf_sys::qpdf_error_code_e {
        qpdf_sys::qpdf_error_code_e_qpdf_e_success => return Ok(()),
        qpdf_sys::qpdf_error_code_e_qpdf_e_internal => QPdfErrorCode::InternalError,
        qpdf_sys::qpdf_error_code_e_qpdf_e_system => QPdfErrorCode::SystemError,
        qpdf_sys::qpdf_error_code_e_qpdf_e_unsupported => QPdfErrorCode::Unsupported,
        qpdf_sys::qpdf_error_code_e_qpdf_e_password => QPdfErrorCode::InvalidPassword,
        qpdf_sys::qpdf_error_code_e_qpdf_e_damaged_pdf => QPdfErrorCode::DamagedPdf,
        qpdf_sys::qpdf_error_code_e_qpdf_e_pages => QPdfErrorCode::PagesError,
        qpdf_sys::qpdf_error_code_e_qpdf_e_object => QPdfErrorCode::ObjectError,
        _ => QPdfErrorCode::Unknown,
    };
    Err(QPdfError {
        error_code: code,
        description: None,
        position: None,
    })
}

impl Default for QPdfErrorCode {
    fn default() -> Self {
        QPdfErrorCode::Unknown
    }
}

/// QPdfError holds an error code and an optional extra information
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Default)]
pub struct QPdfError {
    pub(crate) error_code: QPdfErrorCode,
    pub(crate) description: Option<String>,
    pub(crate) position: Option<u64>,
}

impl fmt::Display for QPdfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}: {}",
            self.error_code,
            self.description.as_deref().unwrap_or_default()
        )
    }
}

impl std::error::Error for QPdfError {}

impl QPdfError {
    pub fn error_code(&self) -> QPdfErrorCode {
        self.error_code
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn position(&self) -> Option<u64> {
        self.position
    }
}

impl From<NulError> for QPdfError {
    fn from(_: NulError) -> Self {
        QPdfError {
            error_code: QPdfErrorCode::InvalidParameter,
            description: Some("Unexpected null code in the string parameter".to_owned()),
            position: None,
        }
    }
}
