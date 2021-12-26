use std::{cmp::Ordering, ffi::CStr, fmt, ptr, slice};

use crate::{stream::QpdfStreamData, Qpdf, Result};

/// Types of the QPDF objects
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum QpdfObjectType {
    Uninitialized,
    Reserved,
    Null,
    Boolean,
    Integer,
    Real,
    String,
    Name,
    Array,
    Dictionary,
    Stream,
    Operator,
    InlineImage,
}

impl QpdfObjectType {
    fn from_qpdf_enum(obj_t: qpdf_sys::qpdf_object_type_e) -> Self {
        match obj_t {
            qpdf_sys::qpdf_object_type_e_ot_uninitialized => QpdfObjectType::Uninitialized,
            qpdf_sys::qpdf_object_type_e_ot_reserved => QpdfObjectType::Reserved,
            qpdf_sys::qpdf_object_type_e_ot_null => QpdfObjectType::Null,
            qpdf_sys::qpdf_object_type_e_ot_boolean => QpdfObjectType::Boolean,
            qpdf_sys::qpdf_object_type_e_ot_integer => QpdfObjectType::Integer,
            qpdf_sys::qpdf_object_type_e_ot_real => QpdfObjectType::Real,
            qpdf_sys::qpdf_object_type_e_ot_string => QpdfObjectType::String,
            qpdf_sys::qpdf_object_type_e_ot_name => QpdfObjectType::Name,
            qpdf_sys::qpdf_object_type_e_ot_array => QpdfObjectType::Array,
            qpdf_sys::qpdf_object_type_e_ot_dictionary => QpdfObjectType::Dictionary,
            qpdf_sys::qpdf_object_type_e_ot_stream => QpdfObjectType::Stream,
            qpdf_sys::qpdf_object_type_e_ot_operator => QpdfObjectType::Operator,
            qpdf_sys::qpdf_object_type_e_ot_inlineimage => QpdfObjectType::InlineImage,
            _ => panic!("Unexpected object type!"),
        }
    }
}

/// This structure represents a single PDF object with a lifetime bound to the owning `Qpdf`.
pub struct QpdfObject<'a> {
    pub(crate) owner: &'a Qpdf,
    pub(crate) inner: qpdf_sys::qpdf_oh,
}

impl<'a> QpdfObject<'a> {
    pub(crate) fn new(owner: &'a Qpdf, inner: qpdf_sys::qpdf_oh) -> Self {
        QpdfObject { owner, inner }
    }

    /// Get this object type
    pub fn get_type(&self) -> QpdfObjectType {
        unsafe { QpdfObjectType::from_qpdf_enum(qpdf_sys::qpdf_oh_get_type_code(self.owner.inner, self.inner)) }
    }

    /// 'Unparse' the object converting it to a binary representation
    pub fn to_binary(&self) -> String {
        unsafe {
            CStr::from_ptr(qpdf_sys::qpdf_oh_unparse_binary(self.owner.inner, self.inner))
                .to_string_lossy()
                .into_owned()
        }
    }

    /// Create indirect object from this one
    pub fn into_indirect(self) -> Self {
        unsafe {
            QpdfObject::new(
                self.owner,
                qpdf_sys::qpdf_make_indirect_object(self.owner.inner, self.inner),
            )
        }
    }

    /// Return true if this is a boolean object
    pub fn is_bool(&self) -> bool {
        unsafe { qpdf_sys::qpdf_oh_is_bool(self.owner.inner, self.inner) != 0 }
    }

    /// Return true if this is a real object
    pub fn is_real(&self) -> bool {
        unsafe { qpdf_sys::qpdf_oh_is_real(self.owner.inner, self.inner) != 0 }
    }

    /// Return true if this is an array object
    pub fn is_array(&self) -> bool {
        unsafe { qpdf_sys::qpdf_oh_is_array(self.owner.inner, self.inner) != 0 }
    }

    /// Return true if this is a name object
    pub fn is_name(&self) -> bool {
        unsafe { qpdf_sys::qpdf_oh_is_name(self.owner.inner, self.inner) != 0 }
    }

    /// Return true if this is a string object
    pub fn is_string(&self) -> bool {
        unsafe { qpdf_sys::qpdf_oh_is_string(self.owner.inner, self.inner) != 0 }
    }

    /// Return true if this is an operator object
    pub fn is_operator(&self) -> bool {
        unsafe { qpdf_sys::qpdf_oh_is_operator(self.owner.inner, self.inner) != 0 }
    }

    /// Return true if this is a null object
    pub fn is_null(&self) -> bool {
        unsafe { qpdf_sys::qpdf_oh_is_null(self.owner.inner, self.inner) != 0 }
    }

    /// Return true if this is a scalar object
    pub fn is_scalar(&self) -> bool {
        unsafe { qpdf_sys::qpdf_oh_is_scalar(self.owner.inner, self.inner) != 0 }
    }

    /// Return true if this is an indirect object
    pub fn is_indirect(&self) -> bool {
        unsafe { qpdf_sys::qpdf_oh_is_indirect(self.owner.inner, self.inner) != 0 }
    }

    /// Return true if this is a dictionary object
    pub fn is_dictionary(&self) -> bool {
        unsafe { qpdf_sys::qpdf_oh_is_dictionary(self.owner.inner, self.inner) != 0 }
    }

    /// Return true if the object is initialized
    pub fn is_initialized(&self) -> bool {
        unsafe { qpdf_sys::qpdf_oh_is_initialized(self.owner.inner, self.inner) != 0 }
    }

    /// Return true if the object contains an inline image
    pub fn is_inline_image(&self) -> bool {
        unsafe { qpdf_sys::qpdf_oh_is_inline_image(self.owner.inner, self.inner) != 0 }
    }

    /// Return true if this is a stream object
    pub fn is_stream(&self) -> bool {
        unsafe { qpdf_sys::qpdf_oh_is_stream(self.owner.inner, self.inner) != 0 }
    }

    /// Get boolean value
    pub fn as_bool(&self) -> bool {
        unsafe { qpdf_sys::qpdf_oh_get_bool_value(self.owner.inner, self.inner) != 0 }
    }

    /// Get name value
    pub fn as_name(&self) -> String {
        unsafe {
            CStr::from_ptr(qpdf_sys::qpdf_oh_get_name(self.owner.inner, self.inner))
                .to_string_lossy()
                .into_owned()
        }
    }

    /// Get string value
    pub fn as_string(&self) -> String {
        unsafe {
            CStr::from_ptr(qpdf_sys::qpdf_oh_get_utf8_value(self.owner.inner, self.inner))
                .to_string_lossy()
                .into_owned()
        }
    }

    /// Get binary string value
    pub fn as_binary_string(&self) -> Vec<u8> {
        unsafe {
            let mut length = 0;
            let data = qpdf_sys::qpdf_oh_get_binary_string_value(self.owner.inner, self.inner, &mut length);
            slice::from_raw_parts(data as *const u8, length as _).to_vec()
        }
    }

    /// Get contents from the page object
    pub fn get_page_content_data(&self) -> Result<QpdfStreamData> {
        unsafe {
            let mut len = 0;
            let mut buffer = ptr::null_mut();
            qpdf_sys::qpdf_oh_get_page_content_data(self.owner.inner, self.inner, &mut buffer, &mut len);
            self.owner.last_error_or_then(|| QpdfStreamData::new(buffer, len as _))
        }
    }

    /// Get ID of the indirect object
    pub fn get_id(&self) -> u32 {
        unsafe { qpdf_sys::qpdf_oh_get_object_id(self.owner.inner, self.inner) as _ }
    }

    /// Get generation of the indirect object
    pub fn get_generation(&self) -> u32 {
        unsafe { qpdf_sys::qpdf_oh_get_generation(self.owner.inner, self.inner) as _ }
    }
}

impl<'a> AsRef<QpdfObject<'a>> for QpdfObject<'a> {
    fn as_ref(&self) -> &QpdfObject<'a> {
        self
    }
}

impl<'a> fmt::Debug for QpdfObject<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QpdfObject {{ {} }}", self.to_string())
    }
}
impl<'a> Clone for QpdfObject<'a> {
    fn clone(&self) -> Self {
        unsafe {
            QpdfObject {
                owner: self.owner,
                inner: qpdf_sys::qpdf_oh_new_object(self.owner.inner, self.inner),
            }
        }
    }
}

impl<'a> PartialEq for QpdfObject<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<'a> PartialOrd for QpdfObject<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

impl<'a> Drop for QpdfObject<'a> {
    fn drop(&mut self) {
        unsafe {
            qpdf_sys::qpdf_oh_release(self.owner.inner, self.inner);
        }
    }
}

impl<'a> fmt::Display for QpdfObject<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            write!(
                f,
                "{}",
                CStr::from_ptr(qpdf_sys::qpdf_oh_unparse(self.owner.inner, self.inner)).to_string_lossy()
            )
        }
    }
}
