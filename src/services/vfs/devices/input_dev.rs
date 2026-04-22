extern crate alloc;
// use crate::services::vfs::vnode::{VnodeType, Vnode};
use crate::drivers::input::virtio::InputEvent;
use crate::libkern::dmesg::kernel_log;

pub struct VirtIOInputDev;

impl VirtIOInputDev {
    pub fn read() -> Result<InputEvent, &'static str> {
        if let Some(event) = crate::drivers::input::virtio::INPUT_RING_BUFFER.lock().pop() {
            return Ok(event);
        }
        Err("EAGAIN: Resource temporarily unavailable")
    }
}

pub fn input_dev_init() {
    kernel_log("VFS", "Mapping /dev/input0 to VirtIO-Input device node.");
    // In a real VFS, we would create a Vnode and register it in the namecache under /dev/
}
