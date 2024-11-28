#![allow(clippy::manual_c_str_literals)]
#![doc = include_str!("../README.md")]

use std::{
    cell::RefCell,
    collections::HashSet,
    ffi::{CStr, CString},
    fmt,
    hash::{self, Hasher},
    path::Path,
    ptr,
    rc::Rc,
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

pub type Result<T> = std::result::Result<T, QPdfError>;

struct Handle {
    handle: qpdf_sys::qpdf_data,
    buffer: Vec<u8>,
}

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            qpdf_sys::qpdf_cleanup(&mut self.handle);
        }
    }
}

/// QPdf is a data structure which represents a PDF file
#[derive(Clone)]
pub struct QPdf {
    inner: Rc<Handle>,
    foreign: Rc<RefCell<HashSet<QPdf>>>,
}

impl hash::Hash for QPdf {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.handle.hash(state)
    }
}

impl PartialEq<Self> for QPdf {
    fn eq(&self, other: &Self) -> bool {
        self.inner.handle.eq(&other.inner.handle)
    }
}

impl Eq for QPdf {}

impl fmt::Debug for QPdf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QPDF version {}", QPdf::library_version())
    }
}

impl QPdf {
    pub(crate) fn inner(&self) -> qpdf_sys::qpdf_data {
        self.inner.handle
    }

    fn wrap_ffi_call<F, R>(self: &QPdf, f: F) -> Result<()>
    where
        F: FnOnce() -> R,
    {
        f();
        self.last_error_or_then(|| ())
    }

    fn last_error_or_then<F, T>(self: &QPdf, f: F) -> Result<T>
    where
        F: FnOnce() -> T,
    {
        unsafe {
            if qpdf_sys::qpdf_has_error(self.inner()) == 0 {
                return Ok(f());
            }

            let qpdf_error = qpdf_sys::qpdf_get_error(self.inner());
            let code = qpdf_sys::qpdf_get_error_code(self.inner(), qpdf_error);

            match error_or_ok(code) {
                Ok(_) => Ok(f()),
                Err(e) => {
                    let error_detail = qpdf_sys::qpdf_get_error_message_detail(self.inner(), qpdf_error);

                    let description = if !error_detail.is_null() {
                        Some(CStr::from_ptr(error_detail).to_string_lossy().into_owned())
                    } else {
                        None
                    };

                    let position = qpdf_sys::qpdf_get_error_file_position(self.inner(), qpdf_error);

                    Err(QPdfError {
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

    fn new() -> QPdf {
        Self::new_with_buffer(Vec::new())
    }

    fn new_with_buffer(buffer: Vec<u8>) -> QPdf {
        unsafe {
            let inner = qpdf_sys::qpdf_init();
            qpdf_sys::qpdf_set_suppress_warnings(inner, true.into());
            qpdf_sys::qpdf_silence_errors(inner);
            QPdf {
                inner: Rc::new(Handle { handle: inner, buffer }),
                foreign: Rc::new(RefCell::new(HashSet::new())),
            }
        }
    }

    /// Create an empty PDF
    pub fn empty() -> QPdf {
        let qpdf = QPdf::new();
        unsafe {
            qpdf_sys::qpdf_empty_pdf(qpdf.inner());
        }
        qpdf
    }

    fn do_read_file(self: &QPdf, path: &Path, password: Option<&str>) -> Result<()> {
        let filename = CString::new(path.to_string_lossy().as_ref())?;
        let password = password.and_then(|p| CString::new(p).ok());

        let raw_password = password.as_ref().map(|p| p.as_ptr()).unwrap_or_else(ptr::null);

        self.wrap_ffi_call(|| unsafe { qpdf_sys::qpdf_read(self.inner(), filename.as_ptr(), raw_password) })
    }

    fn do_read_from_memory(self: &QPdf, password: Option<&str>) -> Result<()> {
        let password = password.and_then(|p| CString::new(p).ok());

        let raw_password = password.as_ref().map(|p| p.as_ptr()).unwrap_or_else(ptr::null);

        self.wrap_ffi_call(|| unsafe {
            qpdf_sys::qpdf_read_memory(
                self.inner(),
                b"memory\0".as_ptr() as _,
                self.inner.buffer.as_ptr() as _,
                self.inner.buffer.len() as _,
                raw_password,
            );
        })
    }

    /// Read PDF from the file
    pub fn read<P: AsRef<Path>>(path: P) -> Result<QPdf> {
        let qpdf = QPdf::new();
        qpdf.do_read_file(path.as_ref(), None)?;
        Ok(qpdf)
    }

    /// Load encrypted PDF from the file
    pub fn read_encrypted<P: AsRef<Path>>(path: P, password: &str) -> Result<QPdf> {
        let qpdf = QPdf::new();
        qpdf.do_read_file(path.as_ref(), Some(password))?;
        Ok(qpdf)
    }

    /// Read PDF from memory
    pub fn read_from_memory<T: AsRef<[u8]>>(buffer: T) -> Result<QPdf> {
        let qpdf = QPdf::new_with_buffer(buffer.as_ref().into());
        qpdf.do_read_from_memory(None)?;
        Ok(qpdf)
    }

    /// Read encrypted PDF from memory
    pub fn read_from_memory_encrypted<T: AsRef<[u8]>>(buffer: T, password: &str) -> Result<QPdf> {
        let qpdf = QPdf::new_with_buffer(buffer.as_ref().into());
        qpdf.do_read_from_memory(Some(password))?;
        Ok(qpdf)
    }

    /// Return QPdfWriter used to write PDF to file or memory
    pub fn writer(self: &QPdf) -> QPdfWriter {
        QPdfWriter::new(self.clone())
    }

    /// Check PDF for errors
    pub fn check_pdf(self: &QPdf) -> Result<()> {
        self.wrap_ffi_call(|| unsafe { qpdf_sys::qpdf_check_pdf(self.inner()) })
    }

    /// Enable or disable automatic PDF recovery
    pub fn enable_recovery(self: &QPdf, flag: bool) {
        unsafe { qpdf_sys::qpdf_set_attempt_recovery(self.inner(), flag.into()) }
    }

    /// Enable or disable xref streams ignorance
    pub fn ignore_xref_streams(self: &QPdf, flag: bool) {
        unsafe { qpdf_sys::qpdf_set_ignore_xref_streams(self.inner(), flag.into()) }
    }

    /// Get PDF version as a string
    pub fn get_pdf_version(self: &QPdf) -> String {
        unsafe {
            let version = qpdf_sys::qpdf_get_pdf_version(self.inner());
            if version.is_null() {
                String::new()
            } else {
                CStr::from_ptr(version).to_string_lossy().into_owned()
            }
        }
    }

    /// Get PDF extension level
    pub fn get_pdf_extension_level(self: &QPdf) -> u32 {
        unsafe { qpdf_sys::qpdf_get_pdf_extension_level(self.inner()) as _ }
    }

    /// Return true if PDF is linearized
    pub fn is_linearized(self: &QPdf) -> bool {
        unsafe { qpdf_sys::qpdf_is_linearized(self.inner()) != 0 }
    }

    /// Return true if PDF is encrypted
    pub fn is_encrypted(self: &QPdf) -> bool {
        unsafe { qpdf_sys::qpdf_is_encrypted(self.inner()) != 0 }
    }

    /// Add a page object to PDF. The `first` parameter indicates whether to prepend or append it.
    pub fn add_page<T: AsRef<QPdfObject>>(self: &QPdf, new_page: T, first: bool) -> Result<()> {
        if new_page.as_ref().owner.inner() != self.inner() {
            self.foreign.borrow_mut().insert(new_page.as_ref().owner.clone());
        }
        self.wrap_ffi_call(|| unsafe {
            qpdf_sys::qpdf_add_page(
                self.inner(),
                new_page.as_ref().owner.inner(),
                new_page.as_ref().inner,
                first.into(),
            )
        })
    }

    /// Add a page object to PDF before or after a specified `ref_page`. A page may belong to another PDF.
    pub fn add_page_at<N, R>(self: &QPdf, new_page: N, before: bool, ref_page: R) -> Result<()>
    where
        N: AsRef<QPdfObject>,
        R: AsRef<QPdfObject>,
    {
        if new_page.as_ref().owner.inner() != self.inner() {
            self.foreign.borrow_mut().insert(new_page.as_ref().owner.clone());
        }
        self.wrap_ffi_call(|| unsafe {
            qpdf_sys::qpdf_add_page_at(
                self.inner(),
                new_page.as_ref().owner.inner(),
                new_page.as_ref().inner,
                before.into(),
                ref_page.as_ref().inner,
            )
        })
    }

    /// Get number of page objects in the PDF.
    pub fn get_num_pages(self: &QPdf) -> Result<u32> {
        unsafe {
            let n = qpdf_sys::qpdf_get_num_pages(self.inner());
            self.last_error_or_then(|| n as _)
        }
    }

    /// Get a page object from the PDF with a given zero-based index
    pub fn get_page(self: &QPdf, zero_based_index: u32) -> Option<QPdfDictionary> {
        unsafe {
            let oh = qpdf_sys::qpdf_get_page_n(self.inner(), zero_based_index as _);
            self.last_error_or_then(|| ()).ok()?;
            if oh != 0 {
                Some(QPdfObject::new(self.clone(), oh).into())
            } else {
                None
            }
        }
    }

    /// Get all pages from the PDF.
    pub fn get_pages(self: &QPdf) -> Result<Vec<QPdfDictionary>> {
        Ok((0..self.get_num_pages()?).filter_map(|i| self.get_page(i)).collect())
    }

    /// Remove page object from the PDF.
    pub fn remove_page<P: AsRef<QPdfObject>>(self: &QPdf, page: P) -> Result<()> {
        self.foreign.borrow_mut().remove(&page.as_ref().owner);
        self.wrap_ffi_call(|| unsafe { qpdf_sys::qpdf_remove_page(self.inner(), page.as_ref().inner) })
    }

    /// Parse textual representation of PDF object.
    pub fn parse_object(self: &QPdf, object: &str) -> Result<QPdfObject> {
        unsafe {
            let s = CString::new(object)?;
            let oh = qpdf_sys::qpdf_oh_parse(self.inner(), s.as_ptr());
            self.last_error_or_then(|| QPdfObject::new(self.clone(), oh))
        }
    }

    /// Get trailer object.
    pub fn get_trailer(self: &QPdf) -> Option<QPdfDictionary> {
        let oh = unsafe { qpdf_sys::qpdf_get_trailer(self.inner()) };
        self.last_error_or_then(|| ()).ok()?;
        let obj = QPdfObject::new(self.clone(), oh);
        if obj.get_type() != QPdfObjectType::Uninitialized && obj.get_type() != QPdfObjectType::Null {
            Some(obj.into())
        } else {
            None
        }
    }

    /// Get root object.
    pub fn get_root(self: &QPdf) -> Option<QPdfDictionary> {
        let oh = unsafe { qpdf_sys::qpdf_get_root(self.inner()) };
        self.last_error_or_then(|| ()).ok()?;
        let obj = QPdfObject::new(self.clone(), oh);
        if obj.get_type() != QPdfObjectType::Uninitialized && obj.get_type() != QPdfObjectType::Null {
            Some(obj.into())
        } else {
            None
        }
    }

    /// Find indirect object by object id and generation
    pub fn get_object_by_id(self: &QPdf, obj_id: u32, gen: u32) -> Option<QPdfObject> {
        let oh = unsafe { qpdf_sys::qpdf_get_object_by_id(self.inner(), obj_id as _, gen as _) };
        self.last_error_or_then(|| ()).ok()?;
        let obj = QPdfObject::new(self.clone(), oh);
        if obj.get_type() != QPdfObjectType::Uninitialized && obj.get_type() != QPdfObjectType::Null {
            Some(obj)
        } else {
            None
        }
    }

    /// Replace indirect object by object id and generation
    pub fn replace_object<O: AsRef<QPdfObject>>(self: &QPdf, obj_id: u32, gen: u32, object: O) -> Result<()> {
        self.wrap_ffi_call(|| unsafe {
            qpdf_sys::qpdf_replace_object(self.inner(), obj_id as _, gen as _, object.as_ref().inner)
        })
    }

    /// Create a bool object
    pub fn new_bool(self: &QPdf, value: bool) -> QPdfObject {
        let oh = unsafe { qpdf_sys::qpdf_oh_new_bool(self.inner(), value.into()) };
        QPdfObject::new(self.clone(), oh)
    }

    /// Create a null object
    pub fn new_null(self: &QPdf) -> QPdfObject {
        let oh = unsafe { qpdf_sys::qpdf_oh_new_null(self.inner()) };
        QPdfObject::new(self.clone(), oh)
    }

    /// Create an integer object
    pub fn new_integer(self: &QPdf, value: i64) -> QPdfScalar {
        let oh = unsafe { qpdf_sys::qpdf_oh_new_integer(self.inner(), value) };
        QPdfObject::new(self.clone(), oh).into()
    }

    /// Create a real object from the textual representation
    pub fn new_real_from_string(self: &QPdf, value: &str) -> QPdfScalar {
        let oh = unsafe {
            let value_str = CString::new(value).unwrap();
            qpdf_sys::qpdf_oh_new_real_from_string(self.inner(), value_str.as_ptr())
        };
        QPdfObject::new(self.clone(), oh).into()
    }

    /// Create a real object from the double value
    pub fn new_real(self: &QPdf, value: f64, decimal_places: u32) -> QPdfScalar {
        let oh = unsafe { qpdf_sys::qpdf_oh_new_real_from_double(self.inner(), value, decimal_places as _) };
        QPdfObject::new(self.clone(), oh).into()
    }

    /// Create an empty array object
    pub fn new_array(self: &QPdf) -> QPdfArray {
        let oh = unsafe { qpdf_sys::qpdf_oh_new_array(self.inner()) };
        QPdfObject::new(self.clone(), oh).into()
    }

    /// Create an array object from the iterator
    pub fn new_array_from<I>(self: &QPdf, iter: I) -> QPdfArray
    where
        I: IntoIterator<Item = QPdfObject>,
    {
        let oh = unsafe { qpdf_sys::qpdf_oh_new_array(self.inner()) };
        let array: QPdfArray = QPdfObject::new(self.clone(), oh).into();
        for item in iter.into_iter() {
            array.push(&item);
        }
        array
    }

    /// Create a name object
    pub fn new_name(self: &QPdf, value: &str) -> QPdfObject {
        let oh = unsafe {
            let value_str = CString::new(value).unwrap();
            qpdf_sys::qpdf_oh_new_name(self.inner(), value_str.as_ptr())
        };
        QPdfObject::new(self.clone(), oh)
    }

    /// Create a string object encoded as a PDF string or binary string
    pub fn new_utf8_string(self: &QPdf, value: &str) -> QPdfObject {
        let oh = unsafe {
            let value_str = CString::new(value).unwrap();
            qpdf_sys::qpdf_oh_new_unicode_string(self.inner(), value_str.as_ptr())
        };
        QPdfObject::new(self.clone(), oh)
    }

    /// Create a PDF string object enclosed in parentheses
    pub fn new_string(self: &QPdf, value: &str) -> QPdfObject {
        let oh = unsafe {
            let value_str = CString::new(value).unwrap();
            qpdf_sys::qpdf_oh_new_string(self.inner(), value_str.as_ptr())
        };
        QPdfObject::new(self.clone(), oh)
    }

    /// Create a binary string object enclosed in angle brackets
    pub fn new_binary_string<V: AsRef<[u8]>>(self: &QPdf, value: V) -> QPdfObject {
        let oh = unsafe {
            qpdf_sys::qpdf_oh_new_binary_string(self.inner(), value.as_ref().as_ptr() as _, value.as_ref().len() as _)
        };
        QPdfObject::new(self.clone(), oh)
    }

    /// Create an empty dictionary object
    pub fn new_dictionary(self: &QPdf) -> QPdfDictionary {
        let oh = unsafe { qpdf_sys::qpdf_oh_new_dictionary(self.inner()) };
        QPdfDictionary::new(QPdfObject::new(self.clone(), oh))
    }

    /// Create a dictionary object from the iterator
    pub fn new_dictionary_from<I, S, O>(self: &QPdf, iter: I) -> QPdfDictionary
    where
        I: IntoIterator<Item = (S, O)>,
        S: AsRef<str>,
        O: Into<QPdfObject>,
    {
        let oh = unsafe { qpdf_sys::qpdf_oh_new_dictionary(self.inner()) };
        let dict = QPdfDictionary::new(QPdfObject::new(self.clone(), oh));
        for item in iter.into_iter() {
            dict.set(item.0.as_ref(), item.1.into());
        }
        dict
    }

    /// Create a stream object with the specified contents. The filter and params are not set.
    pub fn new_stream<D: AsRef<[u8]>>(self: &QPdf, data: D) -> QPdfStream {
        let oh = unsafe { qpdf_sys::qpdf_oh_new_stream(self.inner()) };
        let obj: QPdfStream = QPdfObject::new(self.clone(), oh).into();
        obj.replace_data(data, self.new_null(), self.new_null());
        obj
    }

    /// Create a stream object with specified dictionary and contents. The filter and params are not set.
    pub fn new_stream_with_dictionary<I, S, O, T>(self: &QPdf, iter: I, data: T) -> QPdfStream
    where
        I: IntoIterator<Item = (S, O)>,
        S: AsRef<str>,
        O: Into<QPdfObject>,
        T: AsRef<[u8]>,
    {
        let stream = self.new_stream(data.as_ref());
        let dict = stream.get_dictionary();
        for item in iter.into_iter() {
            dict.set(item.0.as_ref(), item.1.into());
        }
        drop(dict);
        stream
    }

    /// Copy object from the foreign PDF
    pub fn copy_from_foreign<F: AsRef<QPdfObject>>(self: &QPdf, foreign: F) -> QPdfObject {
        let oh = unsafe {
            qpdf_sys::qpdf_oh_copy_foreign_object(self.inner(), foreign.as_ref().owner.inner(), foreign.as_ref().inner)
        };
        self.foreign.borrow_mut().insert(foreign.as_ref().owner.clone());
        QPdfObject::new(self.clone(), oh)
    }

    /// Return true if PDF has warnings
    pub fn more_warnings(self: &QPdf) -> bool {
        unsafe { qpdf_sys::qpdf_more_warnings(self.inner()) != 0 }
    }
}
