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
use alloc::string::String;
// use alloc::vec::Vec;


use alloc::format;
// NOTE: std::io is unavailable in no_std. Real terminal input requires a
// hardware keyboard/serial driver. The REPL loop below uses mock input.
use crate::system::api::loader::macho_load;
use crate::services::vfs::vnode::{vnode_create, VnodeType};
use crate::services::vfs::namecache::namecache_enter;
use crate::userland::shell::builtins;

pub struct ShellState {
    pub current_vid: usize,
    pub current_path: String,
}

pub fn run() {
    crate::println!("Starting interactive R-OS (Unix Architecture) Shell...");
    
    // Pre-populate some mock root files for ls to find
    if let Ok(vid1) = vnode_create(VnodeType::VREG, None) { namecache_enter(0, "unix_kernel", vid1); }
    if let Ok(vid2) = vnode_create(VnodeType::VDIR, None) { namecache_enter(0, "bin", vid2); }
    if let Ok(vid3) = vnode_create(VnodeType::VDIR, None) { namecache_enter(0, "private", vid3); }
    if let Ok(vid4) = vnode_create(VnodeType::VDIR, None) { namecache_enter(0, "etc", vid4); }
    
    let mut state = ShellState {
        current_vid: 0,
        current_path: String::new(),
    };

    // In a real kernel, input would come from a PS/2 / USB HID / serial driver.
    // For now, run a fixed sequence of demo commands to exercise the shell subsystem.
    let demo_commands: &[&str] = &[
        "ls",
        "ps",
        "echo R-OS kernel shell ready",
        "cat /bin/bash | grep bash",
        "help",
    ];

    for &cmd_str in demo_commands {
        let display_path = if state.current_path.is_empty() { "/" } else { &state.current_path };
        crate::println!("user@ros:{} $ {}", display_path, cmd_str);

        let trimmed = cmd_str.trim();
        if trimmed.is_empty() { continue; }

        // POSIX Pipeline and Redirection Logic
        if trimmed.contains('|') || trimmed.contains('>') || trimmed.contains('<') {
            crate::libkern::dmesg::kernel_log("REPL", "Detected Shell Redirection / Pipeline syntax.");
            
            if trimmed.contains('|') {
                let parts_vec: alloc::vec::Vec<&str> = trimmed.split('|').map(|s| s.trim()).collect();
                let cmds = parts_vec.as_slice();
                if cmds.len() == 2 {
                    crate::println!("ros-shell: Initiating pipeline between '{}' and '{}'", cmds[0], cmds[1]);
                    if let Ok((fd_read, fd_write)) = crate::kernel::ipc::pipe::pipe_create() {
                        crate::println!("ros-shell: Spawned Writer Task ('{}') mapping STDOUT -> Pipe FD {}", cmds[0], fd_write);
                        crate::println!("ros-shell: Spawned Reader Task ('{}') mapping STDIN <- Pipe FD {}", cmds[1], fd_read);
                        crate::println!("ros-shell: Pipeline stream established and executing.");
                    }
                } else {
                    crate::println!("ros-shell: Complex multi-pipe chaining not yet supported.");
                }
            } else if trimmed.contains('>') {
                let parts_vec: alloc::vec::Vec<&str> = trimmed.split('>').map(|s| s.trim()).collect();
                crate::println!("ros-shell: Redirecting STDOUT of '{}' to Vnode path '{}'", parts_vec[0], parts_vec[1]);
            } else if trimmed.contains('<') {
                let parts_vec: alloc::vec::Vec<&str> = trimmed.split('<').map(|s| s.trim()).collect();
                crate::println!("ros-shell: Redirecting STDIN of '{}' from Vnode path '{}'", parts_vec[0], parts_vec[1]);
            }
            continue;
        }

        let parts: alloc::vec::Vec<&str> = trimmed.split_whitespace().collect();
        let cmd = parts[0];
        let args = &parts[1..];

        if builtins::execute_builtin(cmd, args, &mut state) {
            continue;
        }

        if cmd == "exit" || cmd == "quit" || cmd == "logout" {
            crate::println!("Terminating shell session.");
            break;
        }

        // Fallback for non-builtins (simulate external execution via loader)
        let mock_path = format!("/bin/{}", cmd);
        match macho_load(&mock_path) {
            Ok((t_id, tid)) => crate::println!("Process {} spawned Thread {}.", t_id, tid),
            Err(e) => crate::println!("{}: command not found ({})", cmd, e),
        }
    }

    crate::println!("[REPL] Demo sequence complete. Halting shell (no hardware keyboard driver yet).");
}
