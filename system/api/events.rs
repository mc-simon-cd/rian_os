extern crate alloc;
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
use alloc::string::String;
use alloc::vec::Vec;


use core::sync::atomic::{AtomicUsize, Ordering};
use alloc::sync::Arc;
use spin::Mutex;
use alloc::format;
use alloc::collections::BTreeMap;
use crate::libkern::dmesg::kernel_log;

static NEXT_KQ_ID: AtomicUsize = AtomicUsize::new(1);

// Standard Kqueue/FreeBSD Filter definitions
pub const EVFILT_READ: u16 = 1;
pub const EVFILT_WRITE: u16 = 2;
pub const EVFILT_MACHPORT: u16 = 3;
pub const EVFILT_VNODE: u16 = 4;
pub const EVFILT_PROC: u16 = 5;

// Edge-Triggering Flags
pub const EV_ADD: u32 = 0x0001;
pub const EV_DELETE: u32 = 0x0002;
pub const EV_ENABLE: u32 = 0x0004;
pub const EV_CLEAR: u32 = 0x0020; // Edge-triggered behavior indicator

#[derive(Clone, Debug)]
pub struct Kevent {
    pub ident: usize, // e.g. Vnode ID
    pub filter: u16,
    pub flags: u32,
    pub fflags: u32,
    pub data: i64,
    pub udata: usize,
}

pub struct Kqueue {
    pub id: usize,
    pub registered_events: BTreeMap<String, Kevent>,
    pub ready_list: Vec<Kevent>, // Holds triggered events resolving kevent() waits natively
}

lazy_static::lazy_static! {
    pub static ref KQUEUES: Arc<Mutex<BTreeMap<usize, Kqueue>>> = Arc::new(Mutex::new(BTreeMap::new()));
}

pub fn kqueue_create() -> Result<usize, &'static str> {
    let id = NEXT_KQ_ID.fetch_add(1, Ordering::SeqCst);
    
    let kq = Kqueue {
        id,
        registered_events: BTreeMap::new(),
        ready_list: Vec::new(),
    };
    
    KQUEUES.lock().insert(id, kq);
    kernel_log("KQUEUE", &format!("Allocated Epoll/Kqueue instance (ID: {})", id));
    
    Ok(id)
}

pub fn kevent_register(kq_id: usize, event: Kevent) -> Result<(), &'static str> {
    let mut queues = KQUEUES.lock();
    if let Some(kq) = queues.get_mut(&kq_id) {
        let event_key = format!("{}_{}", event.ident, event.filter);
        
        if (event.flags & EV_DELETE) != 0 {
            kq.registered_events.remove(&event_key);
            kernel_log("KQUEUE", &format!("kq[{}]: Cleared tracking for ident {}", kq_id, event.ident));
        } else if (event.flags & EV_ADD) != 0 {
            kq.registered_events.insert(event_key, event.clone());
            kernel_log("KQUEUE", &format!("kq[{}]: Tracking ident {} (Filter: {})", kq_id, event.ident, event.filter));
        }
        
        Ok(())
    } else {
        Err("EBADF: Bad file descriptor, kqueue missing")
    }
}

// Resolves polling into ready lists (Mock kernel/user kevent retrieval barrier)
pub fn kevent_wait(kq_id: usize) -> Result<Vec<Kevent>, &'static str> {
    let mut queues = KQUEUES.lock();
    if let Some(kq) = queues.get_mut(&kq_id) {
        if kq.ready_list.is_empty() {
            // Under pure blocking I/O, this would suspend the thread.
            // Since we enforce non-blocking native routines here, return clean.
            return Ok(Vec::new());
        }
        
        let pending = kq.ready_list.clone();
        
        // Edge-Triggering Magic: EV_CLEAR purges the pending state automatically
        // requiring a new hardware state transition to fire again.
        kq.ready_list.retain(|e| (e.flags & EV_CLEAR) == 0);
        kq.ready_list.clear(); // Safe clean
        
        Ok(pending)
    } else {
        Err("EBADF: Bad file descriptor, kqueue missing")
    }
}

// Global trait implementing Observer Pattern for VFS structures
pub trait Notify {
    fn trigger_event(ident: usize, filter: u16);
}

pub struct AsyncEventTrigger;

impl Notify for AsyncEventTrigger {
    fn trigger_event(ident: usize, filter: u16) {
        let mut queues = KQUEUES.lock();
        
        let key = format!("{}_{}", ident, filter);
        for (_, kq) in queues.iter_mut() {
            if kq.registered_events.contains_key(&key) {
                let evt = kq.registered_events.get(&key).unwrap().clone();
                // Check if suspended
                if (evt.flags & EV_ADD) != 0 || (evt.flags & EV_ENABLE) != 0 {
                    kq.ready_list.push(evt);
                    kernel_log("KQUEUE", &format!("Edge-Trigger fired! Ident {} successfully alerted KQ {}", ident, kq.id));
                }
            }
        }
    }
}
