use std::{ptr::NonNull, ops::{Deref, DerefMut}};

pub struct SafeNonNull<T> {
    ptr: NonNull<T>
}

unsafe impl<T> Send for SafeNonNull<T> {}
unsafe impl<T> Sync for SafeNonNull<T> {}

impl<T> SafeNonNull<T> {
    pub fn new(x: T) -> Self {
        Self {
            ptr: unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(x))) }
        }
    }

    pub fn drop(&mut self) {
        drop(unsafe { Box::from_raw(self.ptr.as_ptr()) })
    }
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
