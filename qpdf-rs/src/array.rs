use std::fmt;

use crate::{QpdfObject, QpdfObjectLike};

/// QpdfArray wraps a QpdfObject for array-specific operations
pub struct QpdfArray<'a> {
    inner: QpdfObject<'a>,
}

impl<'a> QpdfArray<'a> {
    fn new(inner: QpdfObject<'a>) -> Self {
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

impl<'a> QpdfObjectLike for QpdfArray<'a> {
    fn inner(&self) -> &QpdfObject {
        &self.inner
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

impl<'a> fmt::Display for QpdfArray<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}
