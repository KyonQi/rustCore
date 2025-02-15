use core::cell::{RefCell, RefMut};

/// Wrap the RefCell with sync trait
pub struct UPSafeCell<T> {
    inner: RefCell<T>,
}

unsafe impl<T> Sync for UPSafeCell<T> { }

impl<T> UPSafeCell<T> {
    /// SAFETY: it's unsafe since the user should guarantee that it's only used in uniprocessor
    pub unsafe fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value),
        }
    }

    /// panic if the data has been borrowed
    pub fn exclusive_access(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}
