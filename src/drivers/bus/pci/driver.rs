extern crate alloc;
use crate::drivers::bus::pci::device::PciDevice;
use alloc::vec::Vec;
use spin::Mutex;

pub trait PciDriver {
    fn match_device(&self, vendor_id: u16, device_id: u16, class: u8) -> bool;
    fn init(&self, device: &PciDevice) -> Result<(), &'static str>;
}

lazy_static::lazy_static! {
    pub static ref DRIVERS: Mutex<Vec<alloc::boxed::Box<dyn PciDriver + Send + Sync>>> = Mutex::new(Vec::new());
}

pub fn register_driver(driver: alloc::boxed::Box<dyn PciDriver + Send + Sync>) {
    (*DRIVERS).lock().push(driver);
}

pub fn bind_drivers() {
    let devices = (*super::PCI_DEVICES).lock();
    let drivers = (*DRIVERS).lock();

    for dev in devices.iter() {
        for driver in drivers.iter() {
            if driver.match_device(dev.vendor_id, dev.device_id, dev.class) {
                let _ = driver.init(dev);
            }
        }
    }
}
