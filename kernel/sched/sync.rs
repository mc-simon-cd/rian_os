// -----------------------------------------------------------------------------
// Copyright 2026 simon_projec
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
// -----------------------------------------------------------------------------

use spin::Mutex;
use alloc::vec::Vec;
use alloc::string::String;

pub struct KMutex<T> {
    name: String,
    inner: Mutex<T>,
    wait_queue: Mutex<Vec<u64>>, // PIDs waiting
}

impl<T> KMutex<T> {
    pub fn new(name: &str, data: T) -> Self {
        Self {
            name: String::from(name),
            inner: Mutex::new(data),
            wait_queue: Mutex::new(Vec::new()),
        }
    }

    pub fn lock(&self, pid: u64) -> Result<spin::MutexGuard<'_, T>, &'static str> {
        if let Some(guard) = self.inner.try_lock() {
            return Ok(guard);
        }
        
        let mut queue = self.wait_queue.lock();
        queue.push(pid);
        Err("Mutex Locked, added to WaitQueue")
    }

    pub fn unlock(&self) {
        // spin::MutexGuard dropping handles the actual unlock.
        // This method simulates the legacy logic of waking up waiters.
        let mut queue = self.wait_queue.lock();
        if !queue.is_empty() {
            let next_pid = queue.remove(0);
            crate::libkern::dmesg::kernel_log("SYNC", &alloc::format!("Mutex '{}' awakened PID {}", self.name, next_pid));
        }
    }
}

pub struct KSpinlock {
    name: String,
    inner: Mutex<()>,
}

impl KSpinlock {
    pub fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
            inner: Mutex::new(()),
        }
    }

    pub fn lock(&self) {
        if self.inner.try_lock().is_none() {
            crate::libkern::dmesg::kernel_log("SYNC", &alloc::format!("Spinlock '{}' spinning...", self.name));
        }
        // block until lock is acquired
        let _guard = self.inner.lock();
    }
}
