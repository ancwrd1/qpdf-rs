use std::{cmp::Ordering, ffi::CStr, fmt, ptr, slice};

use crate::{dict::QpdfDictionary, stream::QpdfStreamData, Qpdf, Result, StreamDecodeLevel};

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

    /// 'Unparse' the object converting it to a textual representation
    pub fn to_string(&self) -> String {
        unsafe {
            CStr::from_ptr(qpdf_sys::qpdf_oh_unparse(self.owner.inner, self.inner))
                .to_string_lossy()
                .into_owned()
        }
    }

    /// 'Unparse' the object converting it to a resolved textual representation
    pub fn to_string_resolved(&self) -> String {
        unsafe {
            CStr::from_ptr(qpdf_sys::qpdf_oh_unparse_resolved(self.owner.inner, self.inner))
                .to_string_lossy()
                .into_owned()
        }
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

    /// Get i64 value
    pub fn as_i64(&self) -> i64 {
        unsafe { qpdf_sys::qpdf_oh_get_int_value(self.owner.inner, self.inner) }
    }

    /// Get u64 value
    pub fn as_u64(&self) -> u64 {
        unsafe { qpdf_sys::qpdf_oh_get_uint_value(self.owner.inner, self.inner) }
    }

    /// Get i32 value
    pub fn as_i32(&self) -> i32 {
        unsafe { qpdf_sys::qpdf_oh_get_int_value_as_int(self.owner.inner, self.inner) }
    }

    /// Get u32 value
    pub fn as_u32(&self) -> u32 {
        unsafe { qpdf_sys::qpdf_oh_get_uint_value_as_uint(self.owner.inner, self.inner) }
    }

    /// Get numeric value
    pub fn as_numeric(&self) -> f64 {
        unsafe { qpdf_sys::qpdf_oh_get_numeric_value(self.owner.inner, self.inner) }
    }

    /// Get real value
    pub fn as_real(&self) -> String {
        unsafe {
            CStr::from_ptr(qpdf_sys::qpdf_oh_get_real_value(self.owner.inner, self.inner))
                .to_string_lossy()
                .into_owned()
        }
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

    /// Get stream data
    pub fn get_stream_data(&self, decode_level: StreamDecodeLevel) -> Result<QpdfStreamData> {
        unsafe {
            let mut filtered = 0;
            let mut len = 0;
            let mut buffer = ptr::null_mut();
            qpdf_sys::qpdf_oh_get_stream_data(
                self.owner.inner,
                self.inner,
                decode_level.as_qpdf_enum(),
                &mut filtered,
                &mut buffer,
                &mut len,
            );
            self.owner.last_error_or_then(|| QpdfStreamData::new(buffer, len as _))
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

    /// Replace stream data
    pub fn replace_stream_data<'b, F, P, D>(&self, data: D, filter: F, params: P)
    where
        F: AsRef<QpdfObject<'b>>,
        P: AsRef<QpdfObject<'b>>,
        D: AsRef<[u8]>,
    {
        unsafe {
            qpdf_sys::qpdf_oh_replace_stream_data(
                self.owner.inner,
                self.inner,
                data.as_ref().as_ptr() as _,
                data.as_ref().len() as _,
                filter.as_ref().inner,
                params.as_ref().inner,
            );
        }
    }

    pub fn get_stream_dictionary(&self) -> QpdfDictionary {
        unsafe { QpdfObject::new(self.owner, qpdf_sys::qpdf_oh_get_dict(self.owner.inner, self.inner)).into() }
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

/// QpdfArray wraps a QpdfObject for array-specific operations
pub struct QpdfArray<'a> {
    inner: QpdfObject<'a>,
}

impl<'a> QpdfArray<'a> {
    fn new(inner: QpdfObject<'a>) -> Self {
        QpdfArray { inner }
    }

    /// Return inner object
    pub fn inner(&self) -> &QpdfObject {
        &self.inner
    }

    /// Return string representation of the dictionary
    pub fn to_string(&self) -> String {
        self.inner.to_string()
    }

    /// Convert object into indirect object
    pub fn into_indirect(self) -> Self {
        QpdfArray::new(self.inner.into_indirect())
    }

    /// Get array length
    pub fn len(&self) -> usize {
        unsafe { qpdf_sys::qpdf_oh_get_array_n_items(self.inner.owner.inner, self.inner.inner) as _ }
    }

    /// Return true if array is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return array iterator
    pub fn iter(&self) -> QpdfArrayIterator {
        QpdfArrayIterator { index: 0, inner: self }
    }

    /// Get array item
    pub fn get(&self, index: usize) -> Option<QpdfObject> {
        if index < self.len() {
            Some(unsafe {
                QpdfObject::new(
                    self.inner.owner,
                    qpdf_sys::qpdf_oh_get_array_item(self.inner.owner.inner, self.inner.inner, index as _),
                )
            })
        } else {
            None
        }
    }

    /// Set array item
    pub fn set<I: AsRef<QpdfObject<'a>>>(&mut self, index: usize, item: I) {
        unsafe {
            qpdf_sys::qpdf_oh_set_array_item(
                self.inner.owner.inner,
                self.inner.inner,
                index as _,
                item.as_ref().inner,
            );
        }
    }

    /// Append an item to the array
    pub fn push<I: AsRef<QpdfObject<'a>>>(&self, item: I) {
        unsafe {
            qpdf_sys::qpdf_oh_append_item(self.inner.owner.inner, self.inner.inner, item.as_ref().inner);
        }
    }

    /// Insert an item into array
    pub fn insert<I: AsRef<QpdfObject<'a>>>(&mut self, index: usize, item: I) {
        unsafe {
            qpdf_sys::qpdf_oh_insert_item(
                self.inner.owner.inner,
                self.inner.inner,
                index as _,
                item.as_ref().inner,
            );
        }
    }

    /// Remove array item
    pub fn remove(&mut self, index: usize) {
        unsafe {
            qpdf_sys::qpdf_oh_erase_item(self.inner.owner.inner, self.inner.inner, index as _);
        }
    }
}

impl<'a> From<QpdfObject<'a>> for QpdfArray<'a> {
    fn from(obj: QpdfObject<'a>) -> Self {
        QpdfArray::new(obj)
    }
}

impl<'a> From<QpdfArray<'a>> for QpdfObject<'a> {
    fn from(dict: QpdfArray<'a>) -> Self {
        dict.inner
    }
}

impl<'a> AsRef<QpdfObject<'a>> for QpdfArray<'a> {
    fn as_ref(&self) -> &QpdfObject<'a> {
        &self.inner
    }
}

/// QpdfArray iterator
pub struct QpdfArrayIterator<'a> {
    index: usize,
    inner: &'a QpdfArray<'a>,
}

impl<'a> Iterator for QpdfArrayIterator<'a> {
    type Item = QpdfObject<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.inner.get(self.index);
        self.index += 1;
        item
    }
}
