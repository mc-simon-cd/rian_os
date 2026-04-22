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
use alloc::string::{String, ToString};


use crate::userland::shell::repl::ShellState;
use crate::services::vfs::namecache::{namecache_list, namecache_enter, namecache_lookup, namecache_remove};
use crate::services::vfs::vnode::{vnode_create, VnodeType, vnode_read, vnode_type};
use crate::kernel::sched::task_thread::{task_list, task_kill};
use crate::libkern::dmesg::get_dmesg;

pub fn execute_builtin(cmd: &str, args: &[&str], state: &mut ShellState) -> bool {
    match cmd {
        "cd" => { handle_cd(args, state); true }
        "ls" => { handle_ls(state); true }
        "pwd" => { handle_pwd(state); true }
        "mkdir" => { handle_mkdir(args, state); true }
        "rm" | "rmdir" => { handle_rm(cmd, args, state); true }
        "touch" => { handle_touch(args, state); true }
        "cat" | "more" | "less" | "head" | "tail" => { handle_read(cmd, args, state); true }
        "ps" | "top" | "htop" => { handle_ps(); true }
        "kill" => { handle_kill(args); true }
        "dmesg" | "journalctl" => { handle_dmesg(); true }
        "uname" | "whoami" | "date" => { handle_sysinfo(cmd); true }
        "df" | "du" => { handle_disk(cmd); true }
        "help" => { handle_help(); true }
        _ => false // Not a builtin
    }
}

fn handle_cd(args: &[&str], state: &mut ShellState) {
    if args.is_empty() || args[0] == "/" || args[0] == "~" {
        state.current_vid = 0;
        state.current_path = String::new();
    } else if args[0] == ".." {
        if state.current_vid != 0 {
            state.current_vid = 0;
            state.current_path = String::new();
        }
    } else {
        let target = args[0];
        if let Some(target_vid) = namecache_lookup(state.current_vid, target) {
            if let Some(t_type) = vnode_type(target_vid) {
                match t_type {
                    VnodeType::VDIR => {
                        state.current_vid = target_vid;
                        state.current_path.push('/');
                        state.current_path.push_str(target);
                    }
                    _ => crate::println!("{}: Not a directory", target),
                }
            }
        } else {
            crate::println!("{}: No such file or directory", target);
        }
    }
}

fn handle_ls(state: &ShellState) {
    let items = namecache_list(state.current_vid);
    if items.is_empty() {
        crate::println!("(empty)");
    } else {
        for item in items { crate::print!("{}  ", item); }
        crate::println!();
    }
}

fn handle_pwd(state: &ShellState) {
    if state.current_path.is_empty() { crate::println!("/"); } else { crate::println!("{}", state.current_path); }
}

fn handle_mkdir(args: &[&str], state: &mut ShellState) {
    if args.is_empty() { crate::println!("mkdir: missing operand"); return; }
    for &dir_name in args {
        if namecache_lookup(state.current_vid, dir_name).is_some() {
            crate::println!("mkdir: cannot create directory '{}': File exists", dir_name);
        } else if let Ok(new_vid) = vnode_create(VnodeType::VDIR, None) {
            namecache_enter(state.current_vid, dir_name, new_vid);
        }
    }
}

fn handle_rm(cmd: &str, args: &[&str], state: &ShellState) {
    if args.is_empty() { crate::println!("{}: missing operand", cmd); return; }
    for &target in args {
        if let Err(e) = namecache_remove(state.current_vid, target) {
            crate::println!("{}: cannot remove '{}': {}", cmd, target, e);
        }
    }
}

fn handle_touch(args: &[&str], state: &mut ShellState) {
    if args.is_empty() { crate::println!("touch: missing file operand"); return; }
    for &file_name in args {
        if namecache_lookup(state.current_vid, file_name).is_none() {
            if let Ok(new_vid) = vnode_create(VnodeType::VREG, Some(String::from("UNIX DATA STREAM\nEOF"))) {
                namecache_enter(state.current_vid, file_name, new_vid);
            }
        }
    }
}

fn handle_read(cmd: &str, args: &[&str], state: &ShellState) {
    if args.is_empty() { crate::println!("{}: missing file operand", cmd); return; }
    for &file_name in args {
        if let Some(vid) = namecache_lookup(state.current_vid, file_name) {
            match vnode_read(vid) {
                Ok(data) => crate::println!("{}", data),
                Err(e) => crate::println!("{}: {}: {}", cmd, file_name, e),
            }
        } else {
            crate::println!("{}: {}: No such file or directory", cmd, file_name);
        }
    }
}

fn handle_ps() {
    crate::println!("{:<8} {:<8} {:<8}", "PID", "PPID", "THREADS");
    for (id, ppid, count) in task_list() {
        let ppid_str = ppid.map_or("0".to_string(), |p| p.to_string());
        crate::println!("{:<8} {:<8} {:<8}", id, ppid_str, count);
    }
}

fn handle_kill(args: &[&str]) {
    if args.is_empty() { crate::println!("kill: usage: kill <pid>"); return; }
    if let Ok(pid) = args[0].parse::<usize>() {
        match task_kill(pid) {
            Ok(_) => crate::println!("Sent kill signal to PID {}", pid),
            Err(e) => crate::println!("kill: {}: {}", pid, e),
        }
    } else {
        crate::println!("kill: invalid pid");
    }
}

fn handle_dmesg() {
    for log in get_dmesg() { crate::println!("{}", log); }
}

fn handle_sysinfo(cmd: &str) {
    match cmd {
        "uname" => crate::println!("R-OS Unix Architecture (x86_64)"),
        "whoami" => crate::println!("root"),
        "date" => crate::println!("Mon Feb 23 2026"),
        _ => {}
    }
}

fn handle_disk(_cmd: &str) {
    crate::println!("Filesystem     Size   Used  Avail Capacity  Mounted on");
    crate::println!("/dev/disk0s1   500G    12G   488G     3%    /");
    crate::println!("devfs          212K   212K     0B   100%    /dev");
}

fn handle_help() {
    crate::println!("Unix R-OS Allowed Builtins:");
    crate::println!("  exit, cd, pwd, ls, mkdir, rmdir, rm, touch, cat, head, tail, more, less");
    crate::println!("  ps, top, htop, kill, dmesg, journalctl, uname, whoami, date, df, du, help");
}
