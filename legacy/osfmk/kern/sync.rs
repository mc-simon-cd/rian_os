// Converted legacy/osfmk/kern/sync.R to Rust
use spin::Mutex;
use spin::MutexGuard;

pub struct KMutex<T> {
    inner: Mutex<T>,
}

impl<T> KMutex<T> {
    pub fn new(val: T) -> Self {
        Self { inner: Mutex::new(val) }
    }
    pub fn lock(&self) -> MutexGuard<T> {
        self.inner.lock()
    }
}

pub struct KSpinlock {
    locked: Mutex<bool>,
}

impl KSpinlock {
    pub fn new() -> Self {
        Self { locked: Mutex::new(false) }
    }
    pub fn lock(&self) {
        let mut l = self.locked.lock();
        *l = true;
    }
    pub fn unlock(&self) {
        let mut l = self.locked.lock();
        *l = false;
    }
}
