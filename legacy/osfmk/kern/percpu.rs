// Converted legacy/osfmk/kern/percpu.R to Rust
use spin::Mutex;
use alloc::collections::BTreeMap;
use crate::libkern::dmesg::kernel_log;

pub struct CpuData {
    pub core_id: u32,
    pub status: String,
}

lazy_static::lazy_static! {
    pub static ref PERCPU_DB: Mutex<BTreeMap<u32, CpuData>> = Mutex::new(BTreeMap::new());
}

pub fn percpu_init() {
    let mut db = PERCPU_DB.lock();
    db.insert(0, CpuData { core_id: 0, status: "ONLINE".to_string() });
    db.insert(1, CpuData { core_id: 1, status: "ONLINE".to_string() });
    kernel_log("PERCPU", "Initialized Thread-Local Storage for 2 Cores [Rust Port]");
}
