use core::{
    cell::UnsafeCell,
    sync::atomic::{AtomicBool, Ordering}, hint,
};

pub struct Mutex<T> {
    data: UnsafeCell<T>,
    lock: AtomicBool,
}

unsafe impl<T> Sync for Mutex<T> {}

pub struct MutexGuard<'a, T> {
    data: &'a mut T,
    lock: &'a AtomicBool,
}

impl<T> Mutex<T> {
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
            lock: AtomicBool::new(true),
        }
    }

    pub fn lock(&self) -> MutexGuard<T> {
        while self
            .lock
            .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            hint::spin_loop();
        }

        MutexGuard {
            lock: &self.lock,
            data: unsafe { &mut *self.data.get() },
        }
    }

    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }
}

impl<'a, T> core::ops::Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a, T> core::ops::DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data
    }
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.store(true, Ordering::SeqCst)
    }
}
