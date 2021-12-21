use std::borrow::Cow;
use std::ffi::{CStr, CString, NulError};
use std::ops::Deref;
use std::path::Path;
use std::{fmt, ptr, slice};

/// Error codes returned by QPDF library calls
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[non_exhaustive]
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
    pub filename: Option<String>,
    pub position: Option<u64>,
}

impl From<NulError> for QpdfError {
    fn from(_: NulError) -> Self {
        QpdfError {
            error_code: QpdfErrorCode::InvalidParameter,
            description: None,
            filename: None,
            position: None,
        }
    }
}

pub type Result<T> = std::result::Result<T, QpdfError>;

/// Qpdf is a data structure which represents a PDF file
pub struct Qpdf {
    inner: qpdf_sys::qpdf_data,
}

impl fmt::Debug for Qpdf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QPDF version {}", Qpdf::library_version())
    }
}

impl Qpdf {
    fn last_error<T>(&self) -> Result<T> {
        unsafe {
            let error = qpdf_sys::qpdf_get_error(self.inner);
            Err(self
                .error_or_ok(qpdf_sys::qpdf_get_error_code(self.inner, error) as _)
                .err()
                .unwrap_or_default())
        }
    }

    fn error_or_ok(&self, error: qpdf_sys::QPDF_ERROR_CODE) -> Result<()> {
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
            filename: None,
            position: None,
        })
    }

    /// Get QPDF library version
    pub fn library_version() -> String {
        unsafe {
            CStr::from_ptr(qpdf_sys::qpdf_get_qpdf_version())
                .to_string_lossy()
                .into_owned()
        }
    }

    /// Create an empty PDF
    pub fn new() -> Self {
        unsafe {
            Qpdf {
                inner: qpdf_sys::qpdf_init(),
            }
        }
    }

    fn do_load_file<P>(path: P, password: Option<&str>) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let qpdf = Qpdf::new();
        let filename = CString::new(path.as_ref().to_string_lossy().as_ref())?;
        let password = password.and_then(|p| CString::new(p).ok());

        unsafe {
            qpdf.error_or_ok(qpdf_sys::qpdf_read(
                qpdf.inner,
                filename.as_ptr(),
                password.map(|p| p.as_ptr()).unwrap_or_else(|| ptr::null()),
            ))?;
        }
        Ok(qpdf)
    }

    pub fn do_load_memory(buf: &[u8], password: Option<&str>) -> Result<Self> {
        let qpdf = Qpdf::new();
        let password = password.and_then(|p| CString::new(p).ok());

        unsafe {
            qpdf.error_or_ok(qpdf_sys::qpdf_read_memory(
                qpdf.inner,
                b"memory\0".as_ptr() as _,
                buf.as_ptr() as _,
                buf.len() as _,
                password.map(|p| p.as_ptr()).unwrap_or_else(|| ptr::null()),
            ))?;
        }
        Ok(qpdf)
    }

    /// Load PDF from the file
    pub fn load<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        Qpdf::do_load_file(path, None)
    }

    /// Load encrypted PDF from the file
    pub fn load_encrypted<P>(path: P, password: &str) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        Qpdf::do_load_file(path, Some(password))
    }

    /// Load PDF from memory
    pub fn load_from_memory(buffer: &[u8]) -> Result<Self> {
        Qpdf::do_load_memory(buffer, None)
    }

    /// Load encrypted PDF from memory
    pub fn load_from_memory_encrypted(buffer: &[u8], password: &str) -> Result<Self> {
        Qpdf::do_load_memory(buffer, Some(password))
    }

    /// Save PDF to a file
    pub fn save<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        unsafe {
            let filename = CString::new(path.as_ref().to_string_lossy().as_ref())?;
            self.error_or_ok(qpdf_sys::qpdf_init_write(self.inner, filename.as_ptr()))?;
            self.error_or_ok(qpdf_sys::qpdf_write(self.inner))
        }
    }

    /// Save PDF to a memory and return a reference to it owned by the Qpdf object
    pub fn save_to_memory(&self) -> Result<&[u8]> {
        unsafe {
            self.error_or_ok(qpdf_sys::qpdf_init_write_memory(self.inner))?;
            self.error_or_ok(qpdf_sys::qpdf_write(self.inner))?;
            let buffer = qpdf_sys::qpdf_get_buffer(self.inner);
            let buffer_len = qpdf_sys::qpdf_get_buffer_length(self.inner);
            Ok(slice::from_raw_parts(buffer as *const u8, buffer_len as _))
        }
    }

    /// Check PDF for errors
    pub fn check_pdf(&self) -> Result<()> {
        unsafe { self.error_or_ok(qpdf_sys::qpdf_check_pdf(self.inner)) }
    }

    /// Enable or disable automatic PDF recovery
    pub fn enable_recovery(&self, flag: bool) {
        unsafe { qpdf_sys::qpdf_set_attempt_recovery(self.inner, flag.into()) }
    }

    /// Enable or disable xref streams ignorance
    pub fn ignore_xref_streams(&self, flag: bool) {
        unsafe { qpdf_sys::qpdf_set_ignore_xref_streams(self.inner, flag.into()) }
    }

    /// Enable or disable stream compression
    pub fn compress_streams(&self, flag: bool) {
        unsafe { qpdf_sys::qpdf_set_compress_streams(self.inner, flag.into()) }
    }

    /// Get PDF version as a string
    pub fn get_pdf_version(&self) -> Option<String> {
        unsafe {
            let version = qpdf_sys::qpdf_get_pdf_version(self.inner);
            if version.is_null() {
                None
            } else {
                Some(CStr::from_ptr(version).to_string_lossy().into_owned())
            }
        }
    }

    /// Get PDF extension level
    pub fn get_pdf_extension_level(&self) -> u32 {
        unsafe { qpdf_sys::qpdf_get_pdf_extension_level(self.inner) as _ }
    }

    /// Return true if PDF is linearized
    pub fn is_linearized(&self) -> bool {
        unsafe { qpdf_sys::qpdf_is_linearized(self.inner) != 0 }
    }

    /// Return true if PDF is encrypted
    pub fn is_encrypted(&self) -> bool {
        unsafe { qpdf_sys::qpdf_is_encrypted(self.inner) != 0 }
    }

    /// Add a page object to PDF. The `first` parameter indicates whether to prepend or append it.
    pub fn add_page<'a>(&self, new_page: &'a QpdfObject, first: bool) -> Result<()> {
        unsafe {
            self.error_or_ok(qpdf_sys::qpdf_add_page(
                self.inner,
                new_page.owner.inner,
                new_page.inner,
                first.into(),
            ))?;
        }
        Ok(())
    }

    /// Add a page object to PDF before or after a specified `ref_page`. A page may belong to another PDF.
    pub fn add_page_at<'a>(
        &self,
        new_page: &'a QpdfObject,
        before: bool,
        ref_page: &QpdfObject,
    ) -> Result<()> {
        unsafe {
            self.error_or_ok(qpdf_sys::qpdf_add_page_at(
                self.inner,
                new_page.owner.inner,
                new_page.inner,
                before.into(),
                ref_page.inner,
            ))?;
        }
        Ok(())
    }

    /// Get number of page objects in the PDF.
    pub fn get_num_pages(&self) -> Result<u32> {
        unsafe {
            let n = qpdf_sys::qpdf_get_num_pages(self.inner);
            if n < 0 {
                self.last_error()
            } else {
                Ok(n as _)
            }
        }
    }

    /// Get a page object from the PDF with a given zero-based index
    pub fn get_page(&self, zero_based_index: u32) -> Option<QpdfObject> {
        unsafe {
            let oh = qpdf_sys::qpdf_get_page_n(self.inner, zero_based_index as _);
            if oh != 0 {
                Some(QpdfObject::new(self, oh))
            } else {
                None
            }
        }
    }

    /// Get all pages from the PDF.
    pub fn get_pages(&self) -> Result<Vec<QpdfObject>> {
        Ok((0..self.get_num_pages()?)
            .map(|i| self.get_page(i))
            .flatten()
            .collect())
    }

    /// Remove page object from the PDF.
    pub fn remove_page(&self, page: &QpdfObject) -> Result<()> {
        unsafe { self.error_or_ok(qpdf_sys::qpdf_remove_page(self.inner, page.inner)) }
    }

    /// Parse textual representation of PDF object.
    pub fn parse_object(&self, object: &str) -> Result<QpdfObject> {
        unsafe {
            let s = CString::new(object)?;
            let oh = qpdf_sys::qpdf_oh_parse(self.inner, s.as_ptr());
            if oh != 0 {
                Ok(QpdfObject::new(self, oh))
            } else {
                self.last_error()
            }
        }
    }

    /// Get trailer object.
    pub fn get_trailer(&self) -> Option<QpdfObject> {
        let oh = unsafe { qpdf_sys::qpdf_get_trailer(self.inner) };
        if oh != 0 {
            Some(QpdfObject::new(self, oh))
        } else {
            None
        }
    }

    /// Get root object.
    pub fn get_root(&self) -> Option<QpdfObject> {
        let oh = unsafe { qpdf_sys::qpdf_get_root(self.inner) };
        if oh != 0 {
            Some(QpdfObject::new(self, oh))
        } else {
            None
        }
    }

    /// Find indirect object by object id and generation
    pub fn get_object_by_id(&self, obj_id: u32, gen: u32) -> Option<QpdfObject> {
        let oh = unsafe { qpdf_sys::qpdf_get_object_by_id(self.inner, obj_id as _, gen as _) };
        if oh != 0 {
            Some(QpdfObject::new(self, oh))
        } else {
            None
        }
    }

    /// Create a bool object
    pub fn new_bool(&self, value: bool) -> QpdfObject {
        let oh = unsafe { qpdf_sys::qpdf_oh_new_bool(self.inner, value.into()) };
        QpdfObject::new(self, oh)
    }

    /// Create a null object
    pub fn new_null(&self) -> QpdfObject {
        let oh = unsafe { qpdf_sys::qpdf_oh_new_null(self.inner) };
        QpdfObject::new(self, oh)
    }

    /// Create an integer object
    pub fn new_integer(&self, value: i64) -> QpdfObject {
        let oh = unsafe { qpdf_sys::qpdf_oh_new_integer(self.inner, value) };
        QpdfObject::new(self, oh)
    }

    /// Create a real object from the textual representation
    pub fn new_real_from_string(&self, value: &str) -> QpdfObject {
        let oh = unsafe {
            let value_str = CString::new(value).unwrap();
            qpdf_sys::qpdf_oh_new_real_from_string(self.inner, value_str.as_ptr())
        };
        QpdfObject::new(self, oh)
    }

    /// Create a real object from the double value
    pub fn new_real(&self, value: f64, decimal_places: u32) -> QpdfObject {
        let oh = unsafe {
            qpdf_sys::qpdf_oh_new_real_from_double(self.inner, value, decimal_places as _)
        };
        QpdfObject::new(self, oh)
    }

    /// Create an array object
    pub fn new_array(&self) -> QpdfArray {
        let oh = unsafe { qpdf_sys::qpdf_oh_new_array(self.inner) };
        QpdfArray::new(QpdfObject::new(self, oh))
    }

    /// Create a name object
    pub fn new_name(&self, value: &str) -> QpdfObject {
        let oh = unsafe {
            let value_str = CString::new(value).unwrap();
            qpdf_sys::qpdf_oh_new_name(self.inner, value_str.as_ptr())
        };
        QpdfObject::new(self, oh)
    }

    /// Create a UTF-8 unicode string object encoded as a binary string
    pub fn new_utf8_string(&self, value: &str) -> QpdfObject {
        let oh = unsafe {
            let value_str = CString::new(value).unwrap();
            qpdf_sys::qpdf_oh_new_unicode_string(self.inner, value_str.as_ptr())
        };
        QpdfObject::new(self, oh)
    }

    /// Create a PDF string object enclosed in parentheses
    pub fn new_string(&self, value: &str) -> QpdfObject {
        let oh = unsafe {
            let value_str = CString::new(value).unwrap();
            qpdf_sys::qpdf_oh_new_string(self.inner, value_str.as_ptr())
        };
        QpdfObject::new(self, oh)
    }

    /// Create a binary string object enclosed in angle brackets
    pub fn new_binary_string(&self, value: &[u8]) -> QpdfObject {
        let oh = unsafe {
            qpdf_sys::qpdf_oh_new_binary_string(self.inner, value.as_ptr() as _, value.len() as _)
        };
        QpdfObject::new(self, oh)
    }

    /// Create a dictionary object
    pub fn new_dictionary(&self) -> QpdfDictionary {
        let oh = unsafe { qpdf_sys::qpdf_oh_new_dictionary(self.inner) };
        QpdfDictionary::new(QpdfObject::new(self, oh))
    }

    /// Create a stream object
    pub fn new_stream(&self) -> QpdfObject {
        let oh = unsafe { qpdf_sys::qpdf_oh_new_stream(self.inner) };
        QpdfObject::new(self, oh)
    }

    /// Create an uninitialized object
    pub fn new_uninitialized(&self) -> QpdfObject {
        let oh = unsafe { qpdf_sys::qpdf_oh_new_uninitialized(self.inner) };
        QpdfObject::new(self, oh)
    }
}

impl Default for Qpdf {
    fn default() -> Self {
        Qpdf::new()
    }
}

impl Drop for Qpdf {
    fn drop(&mut self) {
        unsafe {
            qpdf_sys::qpdf_cleanup(&mut self.inner);
        }
    }
}

/// This structure represents a single PDF object with a lifetime bound to the owning `Qpdf`.
pub struct QpdfObject<'a> {
    owner: &'a Qpdf,
    inner: qpdf_sys::qpdf_oh,
}

impl<'a> QpdfObject<'a> {
    fn new(owner: &'a Qpdf, inner: qpdf_sys::qpdf_oh) -> Self {
        QpdfObject { owner, inner }
    }

    /// 'Unparse' the object converting it to textual representation
    pub fn to_string(&self) -> Cow<str> {
        unsafe {
            CStr::from_ptr(qpdf_sys::qpdf_oh_unparse(self.owner.inner, self.inner))
                .to_string_lossy()
        }
    }

    /// Create indirect object from this one
    pub fn make_indirect(&self) -> Self {
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
    pub fn as_real(&self) -> Cow<str> {
        unsafe {
            CStr::from_ptr(qpdf_sys::qpdf_oh_get_real_value(
                self.owner.inner,
                self.inner,
            ))
            .to_string_lossy()
        }
    }

    /// Get name value
    pub fn as_name(&self) -> Cow<str> {
        unsafe {
            CStr::from_ptr(qpdf_sys::qpdf_oh_get_name(self.owner.inner, self.inner))
                .to_string_lossy()
        }
    }

    /// Get string value
    pub fn as_string(&self) -> Cow<str> {
        unsafe {
            CStr::from_ptr(qpdf_sys::qpdf_oh_get_utf8_value(
                self.owner.inner,
                self.inner,
            ))
            .to_string_lossy()
        }
    }

    /// Get binary string value
    pub fn as_binary_string(&self) -> &[u8] {
        unsafe {
            let mut length = 0;
            let data = qpdf_sys::qpdf_oh_get_binary_string_value(
                self.owner.inner,
                self.inner,
                &mut length,
            );
            slice::from_raw_parts(data as *const u8, length as _)
        }
    }

    /// Wrap into QpdfArray
    pub fn into_array(self) -> QpdfArray<'a> {
        QpdfArray::new(self)
    }

    /// Wrap into QpdfDictionary
    pub fn into_dictionary(self) -> QpdfDictionary<'a> {
        QpdfDictionary::new(self)
    }

    /// Get stream data
    pub fn get_stream_data(&self, decode_level: StreamDecodeLevel) -> Result<QpdfStreamData> {
        unsafe {
            let mut filtered = 0;
            let mut len = 0;
            let mut buffer = ptr::null_mut();
            self.owner.error_or_ok(qpdf_sys::qpdf_oh_get_stream_data(
                self.owner.inner,
                self.inner,
                decode_level.as_qpdf_enum(),
                &mut filtered,
                &mut buffer,
                &mut len,
            ))?;
            Ok(QpdfStreamData {
                data: buffer as _,
                len: len as _,
            })
        }
    }

    /// Get contents from the page object
    pub fn get_page_content_data(&self) -> Result<QpdfStreamData> {
        unsafe {
            let mut len = 0;
            let mut buffer = ptr::null_mut();
            self.owner
                .error_or_ok(qpdf_sys::qpdf_oh_get_page_content_data(
                    self.owner.inner,
                    self.inner,
                    &mut buffer,
                    &mut len,
                ))?;
            Ok(QpdfStreamData {
                data: buffer as _,
                len: len as _,
            })
        }
    }

    /// Replace stream data
    pub fn replace_stream_data(&self, data: &[u8], filter: QpdfObject, params: QpdfObject) {
        unsafe {
            qpdf_sys::qpdf_oh_replace_stream_data(
                self.owner.inner,
                self.inner,
                data.as_ptr() as _,
                data.len() as _,
                filter.inner,
                params.inner,
            );
        }
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

impl<'a> Drop for QpdfObject<'a> {
    fn drop(&mut self) {
        unsafe {
            qpdf_sys::qpdf_oh_release(self.owner.inner, self.inner);
        }
    }
}

/// QpdfArray wraps a QpdfObject for array-specific operations
#[non_exhaustive]
pub struct QpdfArray<'a> {
    pub inner: QpdfObject<'a>,
}

impl<'a> QpdfArray<'a> {
    fn new(inner: QpdfObject<'a>) -> Self {
        QpdfArray { inner }
    }

    /// Get array length
    pub fn len(&self) -> usize {
        unsafe {
            qpdf_sys::qpdf_oh_get_array_n_items(self.inner.owner.inner, self.inner.inner) as _
        }
    }

    /// Return array iterator
    pub fn iter(&self) -> QpdfArrayIterator {
        QpdfArrayIterator {
            index: 0,
            inner: self,
        }
    }

    /// Get array item
    pub fn get(&self, index: usize) -> Option<QpdfObject> {
        if index < self.len() {
            Some(unsafe {
                QpdfObject::new(
                    self.inner.owner,
                    qpdf_sys::qpdf_oh_get_array_item(
                        self.inner.owner.inner,
                        self.inner.inner,
                        index as _,
                    ),
                )
            })
        } else {
            None
        }
    }

    /// Set array item
    pub fn set(&mut self, index: usize, item: QpdfObject<'a>) {
        unsafe {
            qpdf_sys::qpdf_oh_set_array_item(
                self.inner.owner.inner,
                self.inner.inner,
                index as _,
                item.inner,
            );
        }
    }

    /// Append an item to the array
    pub fn push(&mut self, item: QpdfObject<'a>) {
        unsafe {
            qpdf_sys::qpdf_oh_append_item(self.inner.owner.inner, self.inner.inner, item.inner);
        }
    }

    /// Insert an item into array
    pub fn insert(&mut self, index: usize, item: QpdfObject<'a>) {
        unsafe {
            qpdf_sys::qpdf_oh_insert_item(
                self.inner.owner.inner,
                self.inner.inner,
                index as _,
                item.inner,
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

/// QpdfDictionary wraps a QpdfObject for dictionary-related operations
#[non_exhaustive]
pub struct QpdfDictionary<'a> {
    pub inner: QpdfObject<'a>,
}

impl<'a> QpdfDictionary<'a> {
    fn new(inner: QpdfObject<'a>) -> Self {
        QpdfDictionary { inner }
    }

    /// Get dictionary element for the specified key
    pub fn get(&self, key: &str) -> Option<QpdfObject> {
        unsafe {
            let key_str = CString::new(key).unwrap();
            let oh = qpdf_sys::qpdf_oh_get_key(
                self.inner.owner.inner,
                self.inner.inner,
                key_str.as_ptr(),
            );
            let obj = QpdfObject::new(self.inner.owner, oh);
            if !obj.is_null() {
                Some(obj)
            } else {
                None
            }
        }
    }

    /// Set dictionary element for the specified key
    pub fn set(&self, key: &str, value: QpdfObject) {
        unsafe {
            let key_str = CString::new(key).unwrap();
            qpdf_sys::qpdf_oh_replace_key(
                self.inner.owner.inner,
                self.inner.inner,
                key_str.as_ptr(),
                value.inner,
            );
        }
    }

    /// Remove dictionary element
    pub fn remove(&self, key: &str) {
        unsafe {
            let key_str = CString::new(key).unwrap();
            qpdf_sys::qpdf_oh_remove_key(
                self.inner.owner.inner,
                self.inner.inner,
                key_str.as_ptr(),
            );
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

/// Stream decoding level
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum StreamDecodeLevel {
    R3pFull,
    R3pLow,
    R3pNone,
}

impl StreamDecodeLevel {
    fn as_qpdf_enum(&self) -> qpdf_sys::qpdf_stream_decode_level_e {
        match self {
            StreamDecodeLevel::R3pFull => qpdf_sys::qpdf_r3_print_e_qpdf_r3p_full,
            StreamDecodeLevel::R3pLow => qpdf_sys::qpdf_r3_print_e_qpdf_r3p_low,
            StreamDecodeLevel::R3pNone => qpdf_sys::qpdf_r3_print_e_qpdf_r3p_none,
        }
    }
}

/// This structure holds an owned stream data.
pub struct QpdfStreamData {
    data: *const u8,
    len: usize,
}

impl QpdfStreamData {
    /// Get data length
    pub fn len(&self) -> usize {
        self.len
    }
}

impl AsRef<[u8]> for QpdfStreamData {
    fn as_ref(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.data, self.len) }
    }
}

impl Deref for QpdfStreamData {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl Drop for QpdfStreamData {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.data as _);
        }
    }
}
