use std::{
    ffi::{CStr, CString},
    fmt, ptr,
};

use crate::{QPdfObject, QPdfObjectLike, QPdfObjectType, QPdfStreamData, Result};

/// QPdfDictionary wraps a QPdfObject for dictionary-related operations
pub struct QPdfDictionary {
    inner: QPdfObject,
}

impl QPdfDictionary {
    pub(crate) fn new(inner: QPdfObject) -> Self {
        QPdfDictionary { inner }
    }

    /// Get contents from the page object
    pub fn get_page_content_data(&self) -> Result<QPdfStreamData> {
        unsafe {
            let mut len = 0;
            let mut buffer = ptr::null_mut();
            qpdf_sys::qpdf_oh_get_page_content_data(self.inner.owner.inner(), self.inner.inner, &mut buffer, &mut len);
            self.inner
                .owner
                .last_error_or_then(|| QPdfStreamData::new(buffer, len as _))
        }
    }

    /// Check whether there is a key in the dictionary
    pub fn has(&self, key: &str) -> bool {
        unsafe {
            let key_str = CString::new(key).unwrap();
            qpdf_sys::qpdf_oh_has_key(self.inner.owner.inner(), self.inner.inner, key_str.as_ptr()) != 0
        }
    }

    /// Get dictionary element for the specified key
    pub fn get(&self, key: &str) -> Option<QPdfObject> {
        unsafe {
            let key_str = CString::new(key).unwrap();
            let oh = qpdf_sys::qpdf_oh_get_key(self.inner.owner.inner(), self.inner.inner, key_str.as_ptr());
            let obj = QPdfObject::new(self.inner.owner.clone(), oh);
            if obj.get_type() != QPdfObjectType::Null {
                Some(obj)
            } else {
                None
            }
        }
    }

    /// Set dictionary element for the specified key
    pub fn set<V: AsRef<QPdfObject>>(&self, key: &str, value: V) {
        unsafe {
            let key_str = CString::new(key).unwrap();
            qpdf_sys::qpdf_oh_replace_key(
                self.inner.owner.inner(),
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
            qpdf_sys::qpdf_oh_remove_key(self.inner.owner.inner(), self.inner.inner, key_str.as_ptr());
        }
    }

    /// Return all keys from the dictionary
    pub fn keys(&self) -> Vec<String> {
        let mut keys = Vec::new();
        unsafe {
            qpdf_sys::qpdf_oh_begin_dict_key_iter(self.inner.owner.inner(), self.inner.inner);
            while qpdf_sys::qpdf_oh_dict_more_keys(self.inner.owner.inner()) != 0 {
                keys.push(
                    CStr::from_ptr(qpdf_sys::qpdf_oh_dict_next_key(self.inner.owner.inner()))
                        .to_string_lossy()
                        .into_owned(),
                );
            }
        }
        keys
    }
}

impl QPdfObjectLike for QPdfDictionary {
    fn as_object(&self) -> &QPdfObject {
        &self.inner
    }
}

impl From<QPdfObject> for QPdfDictionary {
    fn from(obj: QPdfObject) -> Self {
        QPdfDictionary::new(obj)
    }
}

impl From<QPdfDictionary> for QPdfObject {
    fn from(dict: QPdfDictionary) -> Self {
        dict.inner
    }
}

impl AsRef<QPdfObject> for QPdfDictionary {
    fn as_ref(&self) -> &QPdfObject {
        &self.inner
    }
}

impl fmt::Display for QPdfDictionary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}
