use std::fmt;

use crate::{QPdfObject, QPdfObjectLike};

/// QPdfArray wraps a QPdfObject for array-specific operations
pub struct QPdfArray {
    inner: QPdfObject,
}

impl QPdfArray {
    fn new(inner: QPdfObject) -> Self {
        QPdfArray { inner }
    }

    /// Get array length
    pub fn len(&self) -> usize {
        unsafe { qpdf_sys::qpdf_oh_get_array_n_items(self.inner.owner.inner(), self.inner.inner) as _ }
    }

    /// Return true if array is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return array iterator
    pub fn iter(&self) -> QPdfArrayIterator<'_> {
        QPdfArrayIterator { index: 0, inner: self }
    }

    /// Get array item
    pub fn get(&self, index: usize) -> Option<QPdfObject> {
        if index < self.len() {
            Some(unsafe {
                QPdfObject::new(
                    self.inner.owner.clone(),
                    qpdf_sys::qpdf_oh_get_array_item(self.inner.owner.inner(), self.inner.inner, index as _),
                )
            })
        } else {
            None
        }
    }

    /// Set array item
    pub fn set<I: AsRef<QPdfObject>>(&mut self, index: usize, item: I) {
        unsafe {
            qpdf_sys::qpdf_oh_set_array_item(
                self.inner.owner.inner(),
                self.inner.inner,
                index as _,
                item.as_ref().inner,
            );
        }
    }

    /// Append an item to the array
    pub fn push<I: AsRef<QPdfObject>>(&self, item: I) {
        unsafe {
            qpdf_sys::qpdf_oh_append_item(self.inner.owner.inner(), self.inner.inner, item.as_ref().inner);
        }
    }

    /// Insert an item into array
    pub fn insert<I: AsRef<QPdfObject>>(&mut self, index: usize, item: I) {
        unsafe {
            qpdf_sys::qpdf_oh_insert_item(
                self.inner.owner.inner(),
                self.inner.inner,
                index as _,
                item.as_ref().inner,
            );
        }
    }

    /// Remove array item
    pub fn remove(&mut self, index: usize) {
        unsafe {
            qpdf_sys::qpdf_oh_erase_item(self.inner.owner.inner(), self.inner.inner, index as _);
        }
    }
}

impl QPdfObjectLike for QPdfArray {
    fn as_object(&self) -> &QPdfObject {
        &self.inner
    }
}

impl From<QPdfObject> for QPdfArray {
    fn from(obj: QPdfObject) -> Self {
        QPdfArray::new(obj)
    }
}

impl From<QPdfArray> for QPdfObject {
    fn from(dict: QPdfArray) -> Self {
        dict.inner
    }
}

impl AsRef<QPdfObject> for QPdfArray {
    fn as_ref(&self) -> &QPdfObject {
        &self.inner
    }
}

/// QPdfArray iterator
pub struct QPdfArrayIterator<'a> {
    index: usize,
    inner: &'a QPdfArray,
}

impl Iterator for QPdfArrayIterator<'_> {
    type Item = QPdfObject;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.inner.get(self.index);
        self.index += 1;
        item
    }
}

impl fmt::Display for QPdfArray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}
