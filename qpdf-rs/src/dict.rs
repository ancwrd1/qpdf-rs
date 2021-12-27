use std::{
    ffi::{CStr, CString},
    fmt, ptr,
};

use crate::{QpdfObject, QpdfObjectLike, QpdfObjectType, QpdfStreamData, Result};

/// QpdfDictionary wraps a QpdfObject for dictionary-related operations
pub struct QpdfDictionary<'a> {
    inner: QpdfObject<'a>,
}

impl<'a> QpdfDictionary<'a> {
    pub(crate) fn new(inner: QpdfObject<'a>) -> Self {
        QpdfDictionary { inner }
    }

    /// Return inner QpdfObject
    pub fn inner(&self) -> &QpdfObject {
        &self.inner
    }

    /// Get contents from the page object
    pub fn get_page_content_data(&self) -> Result<QpdfStreamData> {
        unsafe {
            let mut len = 0;
            let mut buffer = ptr::null_mut();
            qpdf_sys::qpdf_oh_get_page_content_data(self.inner.owner.inner, self.inner.inner, &mut buffer, &mut len);
            self.inner
                .owner
                .last_error_or_then(|| QpdfStreamData::new(buffer, len as _))
        }
    }

    /// Check whether there is a key in the dictionary
    pub fn has(&self, key: &str) -> bool {
        unsafe {
            let key_str = CString::new(key).unwrap();
            qpdf_sys::qpdf_oh_has_key(self.inner.owner.inner, self.inner.inner, key_str.as_ptr()) != 0
        }
    }

    /// Get dictionary element for the specified key
    pub fn get(&self, key: &str) -> Option<QpdfObject> {
        unsafe {
            let key_str = CString::new(key).unwrap();
            let oh = qpdf_sys::qpdf_oh_get_key(self.inner.owner.inner, self.inner.inner, key_str.as_ptr());
            let obj = QpdfObject::new(self.inner.owner, oh);
            if obj.get_type() != QpdfObjectType::Null {
                Some(obj)
            } else {
                None
            }
        }
    }

    /// Set dictionary element for the specified key
    pub fn set<V: AsRef<QpdfObject<'a>>>(&self, key: &str, value: V) {
        unsafe {
            let key_str = CString::new(key).unwrap();
            qpdf_sys::qpdf_oh_replace_key(
                self.inner.owner.inner,
                self.inner.inner,
                key_str.as_ptr(),
                value.as_ref().inner,
            );
        }
    }

    /// Remove dictionary element
    pub fn remove(&self, key: &str) {
        unsafe {
            let key_str = CString::new(key).unwrap();
            qpdf_sys::qpdf_oh_remove_key(self.inner.owner.inner, self.inner.inner, key_str.as_ptr());
        }
    }

    /// Return all keys from the dictionary
    pub fn keys(&self) -> Vec<String> {
        let mut keys = Vec::new();
        unsafe {
            qpdf_sys::qpdf_oh_begin_dict_key_iter(self.inner.owner.inner, self.inner.inner);
            while qpdf_sys::qpdf_oh_dict_more_keys(self.inner.owner.inner) != 0 {
                keys.push(
                    CStr::from_ptr(qpdf_sys::qpdf_oh_dict_next_key(self.inner.owner.inner))
                        .to_string_lossy()
                        .into_owned(),
                );
            }
        }
        keys
    }
}

impl<'a> QpdfObjectLike for QpdfDictionary<'a> {
    fn inner(&self) -> &QpdfObject {
        &self.inner
    }
}

impl<'a> From<QpdfObject<'a>> for QpdfDictionary<'a> {
    fn from(obj: QpdfObject<'a>) -> Self {
        QpdfDictionary::new(obj)
    }
}

impl<'a> From<QpdfDictionary<'a>> for QpdfObject<'a> {
    fn from(dict: QpdfDictionary<'a>) -> Self {
        dict.inner
    }
}

impl<'a> AsRef<QpdfObject<'a>> for QpdfDictionary<'a> {
    fn as_ref(&self) -> &QpdfObject<'a> {
        &self.inner
    }
}

impl<'a> fmt::Display for QpdfDictionary<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}