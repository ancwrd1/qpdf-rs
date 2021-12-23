use std::{
    cmp::Ordering,
    ffi::{CStr, CString, NulError},
    fmt,
    ops::Deref,
    path::Path,
    ptr, slice,
};

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

fn error_or_ok(error: qpdf_sys::qpdf_error_code_e) -> Result<()> {
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
    fn wrap_ffi_call<F, R>(&self, f: F) -> Result<()>
    where
        F: FnOnce() -> R,
    {
        f();
        self.last_error_or_then(|| ())
    }

    fn last_error_or_then<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce() -> T,
    {
        unsafe {
            if qpdf_sys::qpdf_has_error(self.inner) == 0 {
                return Ok(f());
            }

            let qpdf_error = qpdf_sys::qpdf_get_error(self.inner);
            let code = qpdf_sys::qpdf_get_error_code(self.inner, qpdf_error);

            match error_or_ok(code) {
                Ok(_) => Ok(f()),
                Err(e) => {
                    let error_detail =
                        qpdf_sys::qpdf_get_error_message_detail(self.inner, qpdf_error);

                    let description = if !error_detail.is_null() {
                        Some(CStr::from_ptr(error_detail).to_string_lossy().into_owned())
                    } else {
                        None
                    };

                    let position = qpdf_sys::qpdf_get_error_file_position(self.inner, qpdf_error);

                    Err(QpdfError {
                        description,
                        position: Some(position),
                        ..e
                    })
                }
            }
        }
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
            let inner = qpdf_sys::qpdf_init();
            qpdf_sys::qpdf_set_suppress_warnings(inner, true.into());
            qpdf_sys::qpdf_silence_errors(inner);
            Qpdf { inner }
        }
    }

    fn do_load_file<P>(&self, path: P, password: Option<&str>) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let filename = CString::new(path.as_ref().to_string_lossy().as_ref())?;
        let password = password.and_then(|p| CString::new(p).ok());

        let raw_password = password
            .as_ref()
            .map(|p| p.as_ptr())
            .unwrap_or_else(ptr::null);

        self.wrap_ffi_call(|| unsafe {
            qpdf_sys::qpdf_read(self.inner, filename.as_ptr(), raw_password)
        })
    }

    pub fn do_load_memory(&self, buf: &[u8], password: Option<&str>) -> Result<()> {
        let password = password.and_then(|p| CString::new(p).ok());

        let raw_password = password
            .as_ref()
            .map(|p| p.as_ptr())
            .unwrap_or_else(ptr::null);

        self.wrap_ffi_call(|| unsafe {
            qpdf_sys::qpdf_read_memory(
                self.inner,
                b"memory\0".as_ptr() as _,
                buf.as_ptr() as _,
                buf.len() as _,
                raw_password,
            );
        })
    }

    /// Load PDF from the file
    pub fn load<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let qpdf = Qpdf::new();
        qpdf.do_load_file(path, None)?;
        Ok(qpdf)
    }

    /// Load encrypted PDF from the file
    pub fn load_encrypted<P>(path: P, password: &str) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let qpdf = Qpdf::new();
        qpdf.do_load_file(path, Some(password))?;
        Ok(qpdf)
    }

    /// Load PDF from memory
    pub fn load_from_memory(buffer: &[u8]) -> Result<Self> {
        let qpdf = Qpdf::new();
        qpdf.do_load_memory(buffer, None)?;
        Ok(qpdf)
    }

    /// Load encrypted PDF from memory
    pub fn load_from_memory_encrypted(buffer: &[u8], password: &str) -> Result<Self> {
        let qpdf = Qpdf::new();
        qpdf.do_load_memory(buffer, Some(password))?;
        Ok(qpdf)
    }

    /// Save PDF to a file
    pub fn save<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let filename = CString::new(path.as_ref().to_string_lossy().as_ref())?;
        self.wrap_ffi_call(|| unsafe { qpdf_sys::qpdf_init_write(self.inner, filename.as_ptr()) })?;
        self.wrap_ffi_call(|| unsafe { qpdf_sys::qpdf_write(self.inner) })
    }

    /// Save PDF to a memory and return a reference to it owned by the Qpdf object
    pub fn save_to_memory(&self) -> Result<Vec<u8>> {
        self.wrap_ffi_call(|| unsafe { qpdf_sys::qpdf_init_write_memory(self.inner) })?;
        self.wrap_ffi_call(|| unsafe { qpdf_sys::qpdf_write(self.inner) })?;
        let buffer = unsafe { qpdf_sys::qpdf_get_buffer(self.inner) };
        let buffer_len = unsafe { qpdf_sys::qpdf_get_buffer_length(self.inner) };
        unsafe { Ok(slice::from_raw_parts(buffer as *const u8, buffer_len as _).to_vec()) }
    }

    /// Check PDF for errors
    pub fn check_pdf(&self) -> Result<()> {
        self.wrap_ffi_call(|| unsafe { qpdf_sys::qpdf_check_pdf(self.inner) })
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
    pub fn get_pdf_version(&self) -> String {
        unsafe {
            let version = qpdf_sys::qpdf_get_pdf_version(self.inner);
            if version.is_null() {
                String::new()
            } else {
                CStr::from_ptr(version).to_string_lossy().into_owned()
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
        self.wrap_ffi_call(|| unsafe {
            qpdf_sys::qpdf_add_page(
                self.inner,
                new_page.owner.inner,
                new_page.inner,
                first.into(),
            )
        })
    }

    /// Add a page object to PDF before or after a specified `ref_page`. A page may belong to another PDF.
    pub fn add_page_at<'a>(
        &self,
        new_page: &'a QpdfObject,
        before: bool,
        ref_page: &QpdfObject,
    ) -> Result<()> {
        self.wrap_ffi_call(|| unsafe {
            qpdf_sys::qpdf_add_page_at(
                self.inner,
                new_page.owner.inner,
                new_page.inner,
                before.into(),
                ref_page.inner,
            )
        })
    }

    /// Get number of page objects in the PDF.
    pub fn get_num_pages(&self) -> Result<u32> {
        unsafe {
            let n = qpdf_sys::qpdf_get_num_pages(self.inner);
            self.last_error_or_then(|| n as _)
        }
    }

    /// Get a page object from the PDF with a given zero-based index
    pub fn get_page(&self, zero_based_index: u32) -> Option<QpdfObject> {
        unsafe {
            let oh = qpdf_sys::qpdf_get_page_n(self.inner, zero_based_index as _);
            self.last_error_or_then(|| ()).ok()?;
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
        self.wrap_ffi_call(|| unsafe { qpdf_sys::qpdf_remove_page(self.inner, page.inner) })
    }

    /// Parse textual representation of PDF object.
    pub fn parse_object(&self, object: &str) -> Result<QpdfObject> {
        unsafe {
            let s = CString::new(object)?;
            let oh = qpdf_sys::qpdf_oh_parse(self.inner, s.as_ptr());
            self.last_error_or_then(|| QpdfObject::new(self, oh))
        }
    }

    /// Get trailer object.
    pub fn get_trailer(&self) -> Option<QpdfDictionary> {
        let oh = unsafe { qpdf_sys::qpdf_get_trailer(self.inner) };
        self.last_error_or_then(|| ()).ok()?;
        let obj = QpdfObject::new(self, oh);
        if obj.is_initialized() && !obj.is_null() {
            Some(obj.into())
        } else {
            None
        }
    }

    /// Get root object.
    pub fn get_root(&self) -> Option<QpdfDictionary> {
        let oh = unsafe { qpdf_sys::qpdf_get_root(self.inner) };
        self.last_error_or_then(|| ()).ok()?;
        let obj = QpdfObject::new(self, oh);
        if obj.is_initialized() && !obj.is_null() {
            Some(obj.into())
        } else {
            None
        }
    }

    /// Find indirect object by object id and generation
    pub fn get_object_by_id(&self, obj_id: u32, gen: u32) -> Option<QpdfObject> {
        let oh = unsafe { qpdf_sys::qpdf_get_object_by_id(self.inner, obj_id as _, gen as _) };
        self.last_error_or_then(|| ()).ok()?;
        let obj = QpdfObject::new(self, oh);
        if obj.is_initialized() && !obj.is_null() {
            Some(obj.into())
        } else {
            None
        }
    }

    /// Replace indirect object by object id and generation
    pub fn replace_object(&self, obj_id: u32, gen: u32, object: &QpdfObject) -> Result<()> {
        self.wrap_ffi_call(|| unsafe {
            qpdf_sys::qpdf_replace_object(self.inner, obj_id as _, gen as _, object.inner)
        })
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

    /// Create an empty array object
    pub fn new_array(&self) -> QpdfArray {
        let oh = unsafe { qpdf_sys::qpdf_oh_new_array(self.inner) };
        QpdfObject::new(self, oh).into()
    }

    /// Create an array object from the iterator
    pub fn new_array_from<'a, I>(&self, iter: I) -> QpdfArray
    where
        I: IntoIterator<Item = QpdfObject<'a>>,
    {
        let oh = unsafe { qpdf_sys::qpdf_oh_new_array(self.inner) };
        let array: QpdfArray = QpdfObject::new(self, oh).into();
        for item in iter.into_iter() {
            array.push(&item);
        }
        array
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

    /// Create an empty dictionary object
    pub fn new_dictionary(&self) -> QpdfDictionary {
        let oh = unsafe { qpdf_sys::qpdf_oh_new_dictionary(self.inner) };
        QpdfDictionary::new(QpdfObject::new(self, oh))
    }

    /// Create a dictionary object from the iterator
    pub fn new_dictionary_from<'a, I, S>(&self, iter: I) -> QpdfDictionary
    where
        I: IntoIterator<Item = (S, QpdfObject<'a>)>,
        S: AsRef<str>,
    {
        let oh = unsafe { qpdf_sys::qpdf_oh_new_dictionary(self.inner) };
        let dict = QpdfDictionary::new(QpdfObject::new(self, oh));
        for item in iter.into_iter() {
            dict.set(item.0.as_ref(), &item.1);
        }
        dict
    }

    /// Create a stream object with the specified contents. The filter and params are not set.
    pub fn new_stream(&self, data: &[u8]) -> QpdfObject {
        let oh = unsafe { qpdf_sys::qpdf_oh_new_stream(self.inner) };
        let obj = QpdfObject::new(self, oh);
        obj.replace_stream_data(data, &self.new_null(), &self.new_null());
        obj
    }

    /// Create a stream object with specified dictionary and contents. The filter and params are not set.
    pub fn new_stream_with_dictionary<'a, I, S>(&self, iter: I, data: &[u8]) -> QpdfObject
    where
        I: IntoIterator<Item = (S, QpdfObject<'a>)>,
        S: AsRef<str>,
    {
        let stream = self.new_stream(data);
        let dict = stream.get_stream_dictionary();
        for item in iter.into_iter() {
            dict.set(item.0.as_ref(), &item.1);
        }
        drop(dict);
        stream
    }

    /// Create an uninitialized object
    pub fn new_uninitialized(&self) -> QpdfObject {
        let oh = unsafe { qpdf_sys::qpdf_oh_new_uninitialized(self.inner) };
        QpdfObject::new(self, oh)
    }

    pub fn copy_from_foreign<'f>(&self, foreign: &QpdfObject<'f>) -> QpdfObject {
        let oh = unsafe {
            qpdf_sys::qpdf_oh_copy_foreign_object(self.inner, foreign.owner.inner, foreign.inner)
        };
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
    owner: &'a Qpdf,
    inner: qpdf_sys::qpdf_oh,
}

impl<'a> QpdfObject<'a> {
    fn new(owner: &'a Qpdf, inner: qpdf_sys::qpdf_oh) -> Self {
        QpdfObject { owner, inner }
    }

    /// Get this object type
    pub fn get_type(&self) -> QpdfObjectType {
        unsafe {
            QpdfObjectType::from_qpdf_enum(qpdf_sys::qpdf_oh_get_type_code(
                self.owner.inner,
                self.inner,
            ))
        }
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
            CStr::from_ptr(qpdf_sys::qpdf_oh_unparse_resolved(
                self.owner.inner,
                self.inner,
            ))
            .to_string_lossy()
            .into_owned()
        }
    }

    /// 'Unparse' the object converting it to a binary representation
    pub fn to_binary(&self) -> String {
        unsafe {
            CStr::from_ptr(qpdf_sys::qpdf_oh_unparse_binary(
                self.owner.inner,
                self.inner,
            ))
            .to_string_lossy()
            .into_owned()
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
    pub fn as_real(&self) -> String {
        unsafe {
            CStr::from_ptr(qpdf_sys::qpdf_oh_get_real_value(
                self.owner.inner,
                self.inner,
            ))
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
            CStr::from_ptr(qpdf_sys::qpdf_oh_get_utf8_value(
                self.owner.inner,
                self.inner,
            ))
            .to_string_lossy()
            .into_owned()
        }
    }

    /// Get binary string value
    pub fn as_binary_string(&self) -> Vec<u8> {
        unsafe {
            let mut length = 0;
            let data = qpdf_sys::qpdf_oh_get_binary_string_value(
                self.owner.inner,
                self.inner,
                &mut length,
            );
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
            self.owner.last_error_or_then(|| QpdfStreamData {
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
            qpdf_sys::qpdf_oh_get_page_content_data(
                self.owner.inner,
                self.inner,
                &mut buffer,
                &mut len,
            );
            self.owner.last_error_or_then(|| QpdfStreamData {
                data: buffer as _,
                len: len as _,
            })
        }
    }

    /// Replace stream data
    pub fn replace_stream_data(&self, data: &[u8], filter: &QpdfObject, params: &QpdfObject) {
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

    pub fn get_stream_dictionary(&self) -> QpdfDictionary {
        unsafe {
            QpdfObject::new(
                self.owner,
                qpdf_sys::qpdf_oh_get_dict(self.owner.inner, self.inner),
            )
            .into()
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

    pub fn inner(&self) -> &QpdfObject {
        &self.inner
    }

    /// Get array length
    pub fn len(&self) -> usize {
        unsafe {
            qpdf_sys::qpdf_oh_get_array_n_items(self.inner.owner.inner, self.inner.inner) as _
        }
    }

    /// Return true if array is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
    pub fn set(&mut self, index: usize, item: &QpdfObject<'a>) {
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
    pub fn push(&self, item: &QpdfObject<'a>) {
        unsafe {
            qpdf_sys::qpdf_oh_append_item(self.inner.owner.inner, self.inner.inner, item.inner);
        }
    }

    /// Insert an item into array
    pub fn insert(&mut self, index: usize, item: &QpdfObject<'a>) {
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

impl<'a> From<QpdfObject<'a>> for QpdfArray<'a> {
    fn from(obj: QpdfObject<'a>) -> Self {
        QpdfArray::new(obj)
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

    /// Check whether there is a key in the dictionary
    pub fn has(&self, key: &str) -> bool {
        unsafe {
            let key_str = CString::new(key).unwrap();
            qpdf_sys::qpdf_oh_has_key(self.inner.owner.inner, self.inner.inner, key_str.as_ptr())
                != 0
        }
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
    pub fn set(&self, key: &str, value: &QpdfObject) {
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

impl<'a> From<QpdfObject<'a>> for QpdfDictionary<'a> {
    fn from(obj: QpdfObject<'a>) -> Self {
        QpdfDictionary::new(obj)
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

    /// Return true if data has zero length
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
