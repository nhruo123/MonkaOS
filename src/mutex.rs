#![allow(dead_code)]

use core::{
    cell::UnsafeCell,
    hint,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::x86::{
    cpu_flags::get_cpu_flags,
    interrupts::{disable_interrupt, enable_interrupt},
};

pub struct Mutex<T> {
    data: UnsafeCell<T>,
    lock: AtomicBool,
}

unsafe impl<T> Sync for Mutex<T> {}

pub struct MutexGuard<'a, T> {
    data: &'a mut T,
    lock: &'a AtomicBool,
    old_interrupt_flag: AtomicBool,
}

impl<T> Mutex<T> {
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
            lock: AtomicBool::new(true),
        }
    }

    pub fn lock(&self) -> MutexGuard<T> {
        let old_interrupt_flag = unsafe { get_cpu_flags().interrupt_enabled() };
        unsafe { disable_interrupt() };
        while self
            .lock
            .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            hint::spin_loop();
        }

        unsafe {
            MutexGuard {
                lock: &self.lock,
                data: &mut *self.data.get(),
                old_interrupt_flag: AtomicBool::new(old_interrupt_flag),
            }
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
        self.lock.store(true, Ordering::SeqCst);

        unsafe {
            if self.old_interrupt_flag.load(Ordering::SeqCst) {
                enable_interrupt();
            }
        }
    }
}
