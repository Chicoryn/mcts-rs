use std::{ptr::NonNull, ops::{Deref, DerefMut}};

#[repr(transparent)]
#[derive(Debug)]
pub struct SafeNonNull<T> {
    ptr: NonNull<T>
}

unsafe impl<T> Send for SafeNonNull<T> {}
unsafe impl<T> Sync for SafeNonNull<T> {}

impl<T> SafeNonNull<T> {
    pub(super) fn new(x: T) -> Self {
        Self::from_raw(Box::into_raw(Box::new(x)))
    }

    pub(super) fn from_raw(x: *mut T) -> Self {
        Self {
            ptr: unsafe { NonNull::new_unchecked(x) }
        }
    }

    pub(super) fn into_raw(&self) -> *mut T {
        self.ptr.as_ptr()
    }

    pub(super) fn drop(&self) {
        drop(unsafe { Box::from_raw(self.ptr.as_ptr()) })
    }
}

impl<T> Copy for SafeNonNull<T> {
    // pass
}

impl<T> Clone for SafeNonNull<T> {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr
        }
    }
}

impl<T> Deref for SafeNonNull<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> DerefMut for SafeNonNull<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T> PartialEq for SafeNonNull<T> {
    fn eq(&self, rhs: &Self) -> bool {
        self.ptr.eq(&rhs.ptr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_create_non_null() {
        assert!(!SafeNonNull::new(()).into_raw().is_null());
    }

    #[test]
    fn clone_copy_ptr() {
        let original = SafeNonNull::new(());
        let clone = original.clone();

        assert_eq!(original.into_raw(), clone.into_raw());
        assert_eq!(original, clone);
    }

    #[test]
    fn from_raw_set_ptr() {
        let ptr = Box::into_raw(Box::new(()));

        assert_eq!(SafeNonNull::from_raw(ptr).into_raw(), ptr);
    }
}
