use std::{ops::Deref, slice};

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
