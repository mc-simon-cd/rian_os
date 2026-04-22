use core::marker::PhantomData;
use ::x86_64::instructions::port::{Port as X86Port, PortWrite, PortRead};

pub struct Port<T> {
    port: X86Port<T>,
    _phantom: PhantomData<T>,
}

impl<T: PortRead + PortWrite> Port<T> {
    pub const fn new(port: u16) -> Self {
        Self {
            port: X86Port::new(port),
            _phantom: PhantomData,
        }
    }

    pub unsafe fn write(&mut self, value: T) {
        self.port.write(value);
    }

    pub unsafe fn read(&mut self) -> T {
        self.port.read()
    }
}
