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
use alloc::string::{String, ToString};
use alloc::collections::BTreeMap;
// (Unused imports removed)
use crate::libkern::dmesg::kernel_log;

pub struct NetState {
    pub interfaces: BTreeMap<String, String>,
}

lazy_static::lazy_static! {
    pub static ref SYS_NET: Mutex<NetState> = Mutex::new(NetState {
        interfaces: {
            let mut m = BTreeMap::new();
            m.insert("lo".to_string(), "127.0.0.1".to_string());
            m.insert("eth0".to_string(), "192.168.1.10".to_string());
            m
        },
    });
}

pub fn net_init() {
    kernel_log("NET", "Virtual TCP/IP Stack Initialized.");
}

pub fn net_ping(ip: &str) {
    let state = SYS_NET.lock();
    let lo_ip = state.interfaces.get("lo").unwrap();
    let _eth0_ip = state.interfaces.get("eth0").unwrap();

    if ip == "localhost" || ip == lo_ip {
        crate::println!("PING {} ({}): 56 data bytes", ip, lo_ip);
        crate::println!("64 bytes from {}: icmp_seq=0 ttl=64 time=0.038 ms", lo_ip);
        crate::println!("64 bytes from {}: icmp_seq=1 ttl=64 time=0.041 ms", lo_ip);
        return;
    }

    crate::println!("PING {}: 56 data bytes", ip);
    // In a real no_std environment, we don't have random/sleep like in R.
    // We'll simulate a few responses.
    for i in 0..3 {
        crate::println!("64 bytes from {}: icmp_seq={} ttl=54 time=15.342 ms", ip, i);
    }
}

pub fn net_netstat() {
    crate::println!("Active Internet connections (servers and established)");
    crate::println!("Proto Recv-Q Send-Q Local Address           Foreign Address         State      ");
    crate::println!("tcp        0      0 0.0.0.0:22              0.0.0.0:*               LISTEN     ");
    crate::println!("tcp        0      0 127.0.0.1:5432          0.0.0.0:*               LISTEN     ");
}
