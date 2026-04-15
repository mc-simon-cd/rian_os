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
extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;


use core::sync::atomic::{AtomicUsize, Ordering};
use alloc::sync::Arc;
use spin::Mutex;
use alloc::collections::VecDeque;
use alloc::format;
use crate::libkern::dmesg::kernel_log;

static NEXT_FD: AtomicUsize = AtomicUsize::new(3);

pub struct UnixPipe {
    pub fd_read: usize,
    pub fd_write: usize,
    pub buffer: VecDeque<u8>,
}

lazy_static::lazy_static! {
    pub static ref PIPES: Arc<Mutex<BTreeMap<usize, UnixPipe>>> = Arc::new(Mutex::new(BTreeMap::new()));
}

pub fn ipc_init() {
    kernel_log("IPC", "Unix Stream Pipes initialized. Mach IPC is stripped out.");
}

pub fn pipe_create() -> Result<(usize, usize), &'static str> {
    let fd_read = NEXT_FD.fetch_add(1, Ordering::SeqCst);
    let fd_write = NEXT_FD.fetch_add(1, Ordering::SeqCst);
    
    let pipe = UnixPipe {
        fd_read,
        fd_write,
        buffer: VecDeque::new(),
    };
    
    // Store pipe by read FD for simple mocking
    PIPES.lock().insert(fd_read, pipe);
    kernel_log("IPC", &format!("Created Unix Pipe (fd_read: {}, fd_write: {})", fd_read, fd_write));
    Ok((fd_read, fd_write))
}

pub fn pipe_write(fd: usize, data: &[u8]) -> Result<usize, &'static str> {
    let mut pipes = PIPES.lock();
    for pipe in pipes.values_mut() {
        if pipe.fd_write == fd {
            pipe.buffer.extend(data.iter());
            kernel_log("IPC", &format!("Wrote {} bytes to pipe fd {}", data.len(), fd));
            return Ok(data.len());
        }
    }
    Err("BAD_FILE_DESCRIPTOR")
}

pub fn pipe_read(fd: usize, len: usize) -> Result<Vec<u8>, &'static str> {
    let mut pipes = PIPES.lock();
    if let Some(pipe) = pipes.get_mut(&fd) {
        let mut out = Vec::new();
        for _ in 0..len {
            if let Some(b) = pipe.buffer.pop_front() {
                out.push(b);
            } else {
                break;
            }
        }
        kernel_log("IPC", &format!("Read {} bytes from pipe fd {}", out.len(), fd));
        return Ok(out);
    }
    Err("BAD_FILE_DESCRIPTOR")
}
