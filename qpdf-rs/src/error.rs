use std::ffi::NulError;
use std::fmt;

use crate::Result;

/// Error codes returned by QPDF library calls
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum QpdfErrorCode {
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
        qpdf_sys::qpdf_error_code_e_qpdf_e_internal => QpdfErrorCode::InternalError,
        qpdf_sys::qpdf_error_code_e_qpdf_e_system => QpdfErrorCode::SystemError,
        qpdf_sys::qpdf_error_code_e_qpdf_e_unsupported => QpdfErrorCode::Unsupported,
        qpdf_sys::qpdf_error_code_e_qpdf_e_password => QpdfErrorCode::InvalidPassword,
        qpdf_sys::qpdf_error_code_e_qpdf_e_damaged_pdf => QpdfErrorCode::DamagedPdf,
        qpdf_sys::qpdf_error_code_e_qpdf_e_pages => QpdfErrorCode::PagesError,
        qpdf_sys::qpdf_error_code_e_qpdf_e_object => QpdfErrorCode::ObjectError,
        _ => QpdfErrorCode::Unknown,
    };
    Err(QpdfError {
        error_code: code,
        description: None,
        position: None,
    })
}

impl Default for QpdfErrorCode {
    fn default() -> Self {
        QpdfErrorCode::Unknown
    }
}

/// QpdfError holds an error code and an optional extra information
#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct QpdfError {
    pub(crate) error_code: QpdfErrorCode,
    pub(crate) description: Option<String>,
    pub(crate) position: Option<u64>,
}

impl fmt::Display for QpdfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}: {}",
            self.error_code,
            self.description.as_deref().unwrap_or_default()
        )
    }
}

impl std::error::Error for QpdfError {}

impl QpdfError {
    pub fn error_code(&self) -> QpdfErrorCode {
        self.error_code
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn position(&self) -> Option<u64> {
        self.position
    }
}

impl From<NulError> for QpdfError {
    fn from(_: NulError) -> Self {
        QpdfError {
            error_code: QpdfErrorCode::InvalidParameter,
            description: Some("Unexpected null code in the string parameter".to_owned()),
            position: None,
        }
    }
}
