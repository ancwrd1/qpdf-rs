use std::{cmp::Ordering, ffi::CStr, fmt, slice};

use crate::QpdfRef;

/// Types of the QPDF objects
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash)]
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

pub trait QpdfObjectLike {
    /// Return inner object
    fn as_object(&self) -> &QpdfObject;

    fn owner(&self) -> QpdfRef {
        self.as_object().owner.clone()
    }

    /// Get this object type
    fn get_type(&self) -> QpdfObjectType {
        self.as_object().get_type()
    }

    /// 'Unparse' the object converting it to a binary representation
    fn to_binary(&self) -> String {
        self.as_object().to_binary()
    }

    /// Return true if this is an operator object
    fn is_operator(&self) -> bool {
        self.as_object().is_operator()
    }

    /// Return true if this is a scalar object
    fn is_scalar(&self) -> bool {
        self.as_object().is_scalar()
    }

    /// Return true if this is an indirect object
    fn is_indirect(&self) -> bool {
        self.as_object().is_indirect()
    }

    /// Get boolean value
    fn as_bool(&self) -> bool {
        self.as_object().as_bool()
    }

    /// Get name value
    fn as_name(&self) -> String {
        self.as_object().as_name()
    }

    /// Get string value
    fn as_string(&self) -> String {
        self.as_object().as_string()
    }

    /// Get binary string value
    fn as_binary_string(&self) -> Vec<u8> {
        self.as_object().as_binary_string()
    }

    /// Get ID of the indirect object
    fn get_id(&self) -> u32 {
        self.as_object().get_id()
    }

    /// Get generation of the indirect object
    fn get_generation(&self) -> u32 {
        self.as_object().get_generation()
    }

    fn into_indirect(self) -> QpdfObject
    where
        Self: Sized + Into<QpdfObject>,
    {
        let obj: QpdfObject = self.into();
        obj.into_indirect()
    }
}

/// This structure represents a single PDF object with a lifetime bound to the owning `Qpdf`.
pub struct QpdfObject {
    pub(crate) owner: QpdfRef,
    pub(crate) inner: qpdf_sys::qpdf_oh,
}

impl QpdfObject {
    pub(crate) fn new(owner: QpdfRef, inner: qpdf_sys::qpdf_oh) -> Self {
        QpdfObject { owner, inner }
    }
}

impl QpdfObjectLike for QpdfObject {
    fn as_object(&self) -> &QpdfObject {
        self
    }

    fn get_type(&self) -> QpdfObjectType {
        unsafe { QpdfObjectType::from_qpdf_enum(qpdf_sys::qpdf_oh_get_type_code(self.owner.inner, self.inner)) }
    }

    fn to_binary(&self) -> String {
        unsafe {
            CStr::from_ptr(qpdf_sys::qpdf_oh_unparse_binary(self.owner.inner, self.inner))
                .to_string_lossy()
                .into_owned()
        }
    }

    fn is_operator(&self) -> bool {
        unsafe { qpdf_sys::qpdf_oh_is_operator(self.owner.inner, self.inner) != 0 }
    }

    fn is_scalar(&self) -> bool {
        unsafe { qpdf_sys::qpdf_oh_is_scalar(self.owner.inner, self.inner) != 0 }
    }

    fn is_indirect(&self) -> bool {
        unsafe { qpdf_sys::qpdf_oh_is_indirect(self.owner.inner, self.inner) != 0 }
    }

    fn as_bool(&self) -> bool {
        unsafe { qpdf_sys::qpdf_oh_get_bool_value(self.owner.inner, self.inner) != 0 }
    }

    fn as_name(&self) -> String {
        unsafe {
            CStr::from_ptr(qpdf_sys::qpdf_oh_get_name(self.owner.inner, self.inner))
                .to_string_lossy()
                .into_owned()
        }
    }

    fn as_string(&self) -> String {
        unsafe {
            CStr::from_ptr(qpdf_sys::qpdf_oh_get_utf8_value(self.owner.inner, self.inner))
                .to_string_lossy()
                .into_owned()
        }
    }

    fn as_binary_string(&self) -> Vec<u8> {
        unsafe {
            let mut length = 0;
            let data = qpdf_sys::qpdf_oh_get_binary_string_value(self.owner.inner, self.inner, &mut length);
            slice::from_raw_parts(data as *const u8, length as _).to_vec()
        }
    }

    fn get_id(&self) -> u32 {
        unsafe { qpdf_sys::qpdf_oh_get_object_id(self.owner.inner, self.inner) as _ }
    }

    fn get_generation(&self) -> u32 {
        unsafe { qpdf_sys::qpdf_oh_get_generation(self.owner.inner, self.inner) as _ }
    }

    /// convert to indirect object
    fn into_indirect(self) -> QpdfObject {
        unsafe {
            QpdfObject::new(
                self.owner.clone(),
                qpdf_sys::qpdf_make_indirect_object(self.owner.inner, self.inner),
            )
        }
    }
}

impl AsRef<QpdfObject> for QpdfObject {
    fn as_ref(&self) -> &QpdfObject {
        self
    }
}

impl fmt::Debug for QpdfObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QpdfObject {{ {} }}", self.to_string())
    }
}
impl Clone for QpdfObject {
    fn clone(&self) -> Self {
        unsafe {
            QpdfObject {
                owner: self.owner.clone(),
                inner: qpdf_sys::qpdf_oh_new_object(self.owner.inner, self.inner),
            }
        }
    }
}

impl PartialEq for QpdfObject {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl PartialOrd for QpdfObject {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

impl Drop for QpdfObject {
    fn drop(&mut self) {
        unsafe {
            qpdf_sys::qpdf_oh_release(self.owner.inner, self.inner);
        }
    }
}

impl fmt::Display for QpdfObject {
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
