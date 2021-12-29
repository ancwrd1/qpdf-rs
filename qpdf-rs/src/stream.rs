use std::{fmt, ops::Deref, ptr, slice};

use crate::{QpdfDictionary, QpdfObject, QpdfObjectLike, Result};

/// Stream decoding level
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash)]
pub enum StreamDecodeLevel {
    None,
    Generalized,
    Specialized,
    All,
}

impl StreamDecodeLevel {
    pub(crate) fn as_qpdf_enum(&self) -> qpdf_sys::qpdf_stream_decode_level_e {
        match self {
            StreamDecodeLevel::None => qpdf_sys::qpdf_stream_decode_level_e_qpdf_dl_none,
            StreamDecodeLevel::Generalized => qpdf_sys::qpdf_stream_decode_level_e_qpdf_dl_generalized,
            StreamDecodeLevel::Specialized => qpdf_sys::qpdf_stream_decode_level_e_qpdf_dl_specialized,
            StreamDecodeLevel::All => qpdf_sys::qpdf_stream_decode_level_e_qpdf_dl_all,
        }
    }
}

/// Object stream mode
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash)]
pub enum ObjectStreamMode {
    Disable,
    Preserve,
    Generate,
}

impl ObjectStreamMode {
    pub(crate) fn as_qpdf_enum(&self) -> qpdf_sys::qpdf_object_stream_e {
        match self {
            ObjectStreamMode::Disable => qpdf_sys::qpdf_object_stream_e_qpdf_o_disable,
            ObjectStreamMode::Preserve => qpdf_sys::qpdf_object_stream_e_qpdf_o_preserve,
            ObjectStreamMode::Generate => qpdf_sys::qpdf_object_stream_e_qpdf_o_generate,
        }
    }
}

/// Object stream mode
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash)]
pub enum StreamDataMode {
    Uncompress,
    Preserve,
    Compress,
}

impl StreamDataMode {
    pub(crate) fn as_qpdf_enum(&self) -> qpdf_sys::qpdf_stream_data_e {
        match self {
            StreamDataMode::Uncompress => qpdf_sys::qpdf_stream_data_e_qpdf_s_uncompress,
            StreamDataMode::Preserve => qpdf_sys::qpdf_stream_data_e_qpdf_s_preserve,
            StreamDataMode::Compress => qpdf_sys::qpdf_stream_data_e_qpdf_s_compress,
        }
    }
}

/// QpdfStream represents a stream object
pub struct QpdfStream {
    inner: QpdfObject,
}

impl QpdfStream {
    pub(crate) fn new(inner: QpdfObject) -> Self {
        QpdfStream { inner }
    }

    /// Replace stream data
    pub fn replace_data<F, P, D>(&self, data: D, filter: F, params: P)
    where
        F: AsRef<QpdfObject>,
        P: AsRef<QpdfObject>,
        D: AsRef<[u8]>,
    {
        unsafe {
            qpdf_sys::qpdf_oh_replace_stream_data(
                self.inner.owner.inner,
                self.inner.inner,
                data.as_ref().as_ptr() as _,
                data.as_ref().len() as _,
                filter.as_ref().inner,
                params.as_ref().inner,
            );
        }
    }

    /// Get stream data
    pub fn get_data(&self, decode_level: StreamDecodeLevel) -> Result<QpdfStreamData> {
        unsafe {
            let mut filtered = 0;
            let mut len = 0;
            let mut buffer = ptr::null_mut();
            qpdf_sys::qpdf_oh_get_stream_data(
                self.inner.owner.inner,
                self.inner.inner,
                decode_level.as_qpdf_enum(),
                &mut filtered,
                &mut buffer,
                &mut len,
            );
            self.inner
                .owner
                .last_error_or_then(|| QpdfStreamData::new(buffer, len as _))
        }
    }

    /// Return a dictionary associated with the stream
    pub fn get_dictionary(&self) -> QpdfDictionary {
        unsafe {
            QpdfObject::new(
                self.inner.owner.clone(),
                qpdf_sys::qpdf_oh_get_dict(self.inner.owner.inner, self.inner.inner),
            )
            .into()
        }
    }
}

impl QpdfObjectLike for QpdfStream {
    fn as_object(&self) -> &QpdfObject {
        &self.inner
    }
}

impl From<QpdfObject> for QpdfStream {
    fn from(obj: QpdfObject) -> Self {
        QpdfStream::new(obj)
    }
}

impl From<QpdfStream> for QpdfObject {
    fn from(dict: QpdfStream) -> Self {
        dict.inner
    }
}

impl AsRef<QpdfObject> for QpdfStream {
    fn as_ref(&self) -> &QpdfObject {
        &self.inner
    }
}

impl fmt::Display for QpdfStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

/// This structure holds an owned stream data.
pub struct QpdfStreamData {
    data: *const u8,
    len: usize,
}

impl QpdfStreamData {
    pub(crate) fn new(data: *const u8, len: usize) -> Self {
        QpdfStreamData { data, len }
    }

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
