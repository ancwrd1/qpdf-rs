use std::{ffi::CStr, fmt};

use crate::{QpdfObject, QpdfObjectLike};

/// QpdfScalar represents scalar objects such as integer and real
pub struct QpdfScalar<'a> {
    inner: QpdfObject<'a>,
}

impl<'a> QpdfScalar<'a> {
    pub(crate) fn new(inner: QpdfObject<'a>) -> Self {
        Self { inner }
    }

    /// Get i64 value
    pub fn as_i64(&self) -> i64 {
        unsafe { qpdf_sys::qpdf_oh_get_int_value(self.inner.owner.inner, self.inner.inner) }
    }

    /// Get u64 value
    pub fn as_u64(&self) -> u64 {
        unsafe { qpdf_sys::qpdf_oh_get_uint_value(self.inner.owner.inner, self.inner.inner) }
    }

    /// Get i32 value
    pub fn as_i32(&self) -> i32 {
        unsafe { qpdf_sys::qpdf_oh_get_int_value_as_int(self.inner.owner.inner, self.inner.inner) }
    }

    /// Get u32 value
    pub fn as_u32(&self) -> u32 {
        unsafe { qpdf_sys::qpdf_oh_get_uint_value_as_uint(self.inner.owner.inner, self.inner.inner) }
    }

    /// Get numeric value
    pub fn as_f64(&self) -> f64 {
        unsafe { qpdf_sys::qpdf_oh_get_numeric_value(self.inner.owner.inner, self.inner.inner) }
    }

    /// Get real value in string format
    pub fn as_real(&self) -> String {
        unsafe {
            CStr::from_ptr(qpdf_sys::qpdf_oh_get_real_value(
                self.inner.owner.inner,
                self.inner.inner,
            ))
            .to_string_lossy()
            .into_owned()
        }
    }
}

impl<'a> QpdfObjectLike for QpdfScalar<'a> {
    fn inner(&self) -> &QpdfObject {
        &self.inner
    }
}

impl<'a> From<QpdfObject<'a>> for QpdfScalar<'a> {
    fn from(obj: QpdfObject<'a>) -> Self {
        QpdfScalar::new(obj)
    }
}

impl<'a> From<QpdfScalar<'a>> for QpdfObject<'a> {
    fn from(dict: QpdfScalar<'a>) -> Self {
        dict.inner
    }
}

impl<'a> AsRef<QpdfObject<'a>> for QpdfScalar<'a> {
    fn as_ref(&self) -> &QpdfObject<'a> {
        &self.inner
    }
}

impl<'a> fmt::Display for QpdfScalar<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}
