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

const MAGAZINE_SIZE: usize = 32;

struct Magazine {
    objects: [*mut u8; MAGAZINE_SIZE],
    top: usize,
}

impl Magazine {
    const fn new() -> Self {
        Self {
            objects: [core::ptr::null_mut(); MAGAZINE_SIZE],
            top: 0,
        }
    }

    fn push(&mut self, obj: *mut u8) -> bool {
        if self.top < MAGAZINE_SIZE {
            self.objects[self.top] = obj;
            self.top += 1;
            true
        } else {
            false
        }
    }

    fn pop(&mut self) -> Option<*mut u8> {
        if self.top > 0 {
            self.top -= 1;
            Some(self.objects[self.top])
        } else {
            None
        }
    }

    fn is_empty(&self) -> bool {
        self.top == 0
    }
}

/// Per-CPU Cache containing magazines for different object sizes.
pub struct CpuLocalCache {
    magazines: [MagazineSet; 7],
}

struct MagazineSet {
    current: Magazine,
    depot: Magazine,
}

impl MagazineSet {
    const fn new() -> Self {
        Self {
            current: Magazine::new(),
            depot: Magazine::new(),
        }
    }
}

impl CpuLocalCache {
    pub const fn new() -> Self {
        Self {
            magazines: [
                MagazineSet::new(), MagazineSet::new(), MagazineSet::new(),
                MagazineSet::new(), MagazineSet::new(), MagazineSet::new(),
                MagazineSet::new(),
            ],
        }
    }

    pub fn alloc(&mut self, size_idx: usize) -> Option<*mut u8> {
        let set = &mut self.magazines[size_idx];
        
        if let Some(obj) = set.current.pop() {
            return Some(obj);
        }

        if !set.depot.is_empty() {
            core::mem::swap(&mut set.current, &mut set.depot);
            return set.current.pop();
        }

        None
    }

    pub fn free(&mut self, size_idx: usize, obj: *mut u8) -> Option<*mut u8> {
        let set = &mut self.magazines[size_idx];

        if set.current.push(obj) {
            return None;
        }

        if set.depot.is_empty() {
            core::mem::swap(&mut set.current, &mut set.depot);
            set.current.push(obj);
            return None;
        }

        Some(obj)
    }
}
