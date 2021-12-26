use std::ffi::NulError;

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
#[non_exhaustive]
pub struct QpdfError {
    pub error_code: QpdfErrorCode,
    pub description: Option<String>,
    pub position: Option<u64>,
}

impl From<NulError> for QpdfError {
    fn from(_: NulError) -> Self {
        QpdfError {
            error_code: QpdfErrorCode::InvalidParameter,
            description: None,
            position: None,
        }
    }
}
