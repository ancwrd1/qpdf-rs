use std::{
    ffi::{CStr, CString},
    fmt,
    path::Path,
    ptr,
};

pub use array::*;
pub use dict::*;
pub use error::*;
pub use object::*;
pub use scalar::*;
pub use stream::*;
pub use writer::*;

pub mod array;
pub mod dict;
pub mod error;
pub mod object;
pub mod scalar;
pub mod stream;
pub mod writer;

const EMPTY_PDF: &[u8] = br#"%PDF-1.3
1 0 obj
<< /Type /Catalog /Pages 2 0 R >>
endobj
2 0 obj
<< /Type /Pages /Kids [] /Count 0 >>
endobj
xref
0 3
0000000000 65535 f
0000000009 00000 n
0000000058 00000 n
trailer << /Size 3 /Root 1 0 R >>
startxref
110
%%EOF
"#;

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
                    let error_detail = qpdf_sys::qpdf_get_error_message_detail(self.inner, qpdf_error);

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

    fn new() -> Self {
        unsafe {
            let inner = qpdf_sys::qpdf_init();
            qpdf_sys::qpdf_set_suppress_warnings(inner, true.into());
            qpdf_sys::qpdf_silence_errors(inner);
            Qpdf { inner }
        }
    }

    /// Create an empty PDF
    pub fn empty() -> Self {
        Qpdf::read_from_memory(EMPTY_PDF).unwrap()
    }

    fn do_read_file(&self, path: &Path, password: Option<&str>) -> Result<()> {
        let filename = CString::new(path.to_string_lossy().as_ref())?;
        let password = password.and_then(|p| CString::new(p).ok());

        let raw_password = password.as_ref().map(|p| p.as_ptr()).unwrap_or_else(ptr::null);

        self.wrap_ffi_call(|| unsafe { qpdf_sys::qpdf_read(self.inner, filename.as_ptr(), raw_password) })
    }

    pub fn do_load_memory(&self, buf: &[u8], password: Option<&str>) -> Result<()> {
        let password = password.and_then(|p| CString::new(p).ok());

        let raw_password = password.as_ref().map(|p| p.as_ptr()).unwrap_or_else(ptr::null);

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

    /// Read PDF from the file
    pub fn read<P: AsRef<Path>>(path: P) -> Result<Self> {
        let qpdf = Qpdf::new();
        qpdf.do_read_file(path.as_ref(), None)?;
        Ok(qpdf)
    }

    /// Load encrypted PDF from the file
    pub fn read_encrypted<P: AsRef<Path>>(path: P, password: &str) -> Result<Self> {
        let qpdf = Qpdf::new();
        qpdf.do_read_file(path.as_ref(), Some(password))?;
        Ok(qpdf)
    }

    /// Read PDF from memory
    pub fn read_from_memory<T: AsRef<[u8]>>(buffer: T) -> Result<Self> {
        let qpdf = Qpdf::new();
        qpdf.do_load_memory(buffer.as_ref(), None)?;
        Ok(qpdf)
    }

    /// Read encrypted PDF from memory
    pub fn read_from_memory_encrypted<T: AsRef<[u8]>>(buffer: T, password: &str) -> Result<Self> {
        let qpdf = Qpdf::new();
        qpdf.do_load_memory(buffer.as_ref(), Some(password))?;
        Ok(qpdf)
    }

    /// Return QpdfWriter used to write PDF to file or memory
    pub fn writer(&self) -> QpdfWriter {
        QpdfWriter::new(self)
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
    pub fn add_page<'a, T: AsRef<QpdfObject<'a>>>(&self, new_page: T, first: bool) -> Result<()> {
        self.wrap_ffi_call(|| unsafe {
            qpdf_sys::qpdf_add_page(
                self.inner,
                new_page.as_ref().owner.inner,
                new_page.as_ref().inner,
                first.into(),
            )
        })
    }

    /// Add a page object to PDF before or after a specified `ref_page`. A page may belong to another PDF.
    pub fn add_page_at<'a, 'b, N, R>(&'a self, new_page: N, before: bool, ref_page: R) -> Result<()>
    where
        N: AsRef<QpdfObject<'b>>,
        R: AsRef<QpdfObject<'a>>,
    {
        self.wrap_ffi_call(|| unsafe {
            qpdf_sys::qpdf_add_page_at(
                self.inner,
                new_page.as_ref().owner.inner,
                new_page.as_ref().inner,
                before.into(),
                ref_page.as_ref().inner,
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
        Ok((0..self.get_num_pages()?).map(|i| self.get_page(i)).flatten().collect())
    }

    /// Remove page object from the PDF.
    pub fn remove_page<'a, P: AsRef<QpdfObject<'a>>>(&self, page: P) -> Result<()> {
        self.wrap_ffi_call(|| unsafe { qpdf_sys::qpdf_remove_page(self.inner, page.as_ref().inner) })
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
            Some(obj)
        } else {
            None
        }
    }

    /// Replace indirect object by object id and generation
    pub fn replace_object<'a, O: AsRef<QpdfObject<'a>>>(&self, obj_id: u32, gen: u32, object: O) -> Result<()> {
        self.wrap_ffi_call(|| unsafe {
            qpdf_sys::qpdf_replace_object(self.inner, obj_id as _, gen as _, object.as_ref().inner)
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
    pub fn new_integer(&self, value: i64) -> QpdfScalar {
        let oh = unsafe { qpdf_sys::qpdf_oh_new_integer(self.inner, value) };
        QpdfObject::new(self, oh).into()
    }

    /// Create a real object from the textual representation
    pub fn new_real_from_string(&self, value: &str) -> QpdfScalar {
        let oh = unsafe {
            let value_str = CString::new(value).unwrap();
            qpdf_sys::qpdf_oh_new_real_from_string(self.inner, value_str.as_ptr())
        };
        QpdfObject::new(self, oh).into()
    }

    /// Create a real object from the double value
    pub fn new_real(&self, value: f64, decimal_places: u32) -> QpdfScalar {
        let oh = unsafe { qpdf_sys::qpdf_oh_new_real_from_double(self.inner, value, decimal_places as _) };
        QpdfObject::new(self, oh).into()
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
    pub fn new_binary_string<V: AsRef<[u8]>>(&self, value: V) -> QpdfObject {
        let oh = unsafe {
            qpdf_sys::qpdf_oh_new_binary_string(self.inner, value.as_ref().as_ptr() as _, value.as_ref().len() as _)
        };
        QpdfObject::new(self, oh)
    }

    /// Create an empty dictionary object
    pub fn new_dictionary(&self) -> QpdfDictionary {
        let oh = unsafe { qpdf_sys::qpdf_oh_new_dictionary(self.inner) };
        QpdfDictionary::new(QpdfObject::new(self, oh))
    }

    /// Create a dictionary object from the iterator
    pub fn new_dictionary_from<'a, I, S, O>(&self, iter: I) -> QpdfDictionary
    where
        I: IntoIterator<Item = (S, O)>,
        S: AsRef<str>,
        O: Into<QpdfObject<'a>>,
    {
        let oh = unsafe { qpdf_sys::qpdf_oh_new_dictionary(self.inner) };
        let dict = QpdfDictionary::new(QpdfObject::new(self, oh));
        for item in iter.into_iter() {
            dict.set(item.0.as_ref(), &item.1.into());
        }
        dict
    }

    /// Create a stream object with the specified contents. The filter and params are not set.
    pub fn new_stream<D: AsRef<[u8]>>(&self, data: D) -> QpdfStream {
        let oh = unsafe { qpdf_sys::qpdf_oh_new_stream(self.inner) };
        let obj: QpdfStream = QpdfObject::new(self, oh).into();
        obj.replace_data(data, &self.new_null(), &self.new_null());
        obj
    }

    /// Create a stream object with specified dictionary and contents. The filter and params are not set.
    pub fn new_stream_with_dictionary<'a, I, S, O, T>(&self, iter: I, data: T) -> QpdfStream
    where
        I: IntoIterator<Item = (S, O)>,
        S: AsRef<str>,
        O: Into<QpdfObject<'a>>,
        T: AsRef<[u8]>,
    {
        let stream = self.new_stream(data.as_ref());
        let dict = stream.get_dictionary();
        for item in iter.into_iter() {
            dict.set(item.0.as_ref(), &item.1.into());
        }
        drop(dict);
        stream
    }

    pub fn copy_from_foreign<'f, F: AsRef<QpdfObject<'f>>>(&self, foreign: F) -> QpdfObject {
        let oh = unsafe {
            qpdf_sys::qpdf_oh_copy_foreign_object(self.inner, foreign.as_ref().owner.inner, foreign.as_ref().inner)
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
