use std::fmt;

use crate::{QpdfObject, QpdfObjectLike};

/// QpdfArray wraps a QpdfObject for array-specific operations
pub struct QpdfArray {
    inner: QpdfObject,
}

impl QpdfArray {
    fn new(inner: QpdfObject) -> Self {
        QpdfArray { inner }
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
                    self.inner.owner.clone(),
                    qpdf_sys::qpdf_oh_get_array_item(self.inner.owner.inner, self.inner.inner, index as _),
                )
            })
        } else {
            None
        }
    }

    /// Set array item
    pub fn set<I: AsRef<QpdfObject>>(&mut self, index: usize, item: I) {
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
    pub fn push<I: AsRef<QpdfObject>>(&self, item: I) {
        unsafe {
            qpdf_sys::qpdf_oh_append_item(self.inner.owner.inner, self.inner.inner, item.as_ref().inner);
        }
    }

    /// Insert an item into array
    pub fn insert<I: AsRef<QpdfObject>>(&mut self, index: usize, item: I) {
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

impl QpdfObjectLike for QpdfArray {
    fn inner(&self) -> &QpdfObject {
        &self.inner
    }
}

impl From<QpdfObject> for QpdfArray {
    fn from(obj: QpdfObject) -> Self {
        QpdfArray::new(obj)
    }
}

impl From<QpdfArray> for QpdfObject {
    fn from(dict: QpdfArray) -> Self {
        dict.inner
    }
}

impl AsRef<QpdfObject> for QpdfArray {
    fn as_ref(&self) -> &QpdfObject {
        &self.inner
    }
}

/// QpdfArray iterator
pub struct QpdfArrayIterator<'a> {
    index: usize,
    inner: &'a QpdfArray,
}

impl<'a> Iterator for QpdfArrayIterator<'a> {
    type Item = QpdfObject;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.inner.get(self.index);
        self.index += 1;
        item
    }
}

impl fmt::Display for QpdfArray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}
