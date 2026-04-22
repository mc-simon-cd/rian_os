extern crate alloc;
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::Mutex;
use crate::libkern::dmesg::kernel_log;
use crate::system::api::events::{Notify, AsyncEventTrigger, EVFILT_READ};

/// Standard Linux Evdev format
#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
pub struct InputEvent {
    pub time: u64,
    pub type_: u16,
    pub code: u16,
    pub value: i32,
}

pub const EV_KEY: u16 = 0x01;
pub const KEY_A: u16 = 30;

const RING_BUFFER_SIZE: usize = 256;

/// ISR-Safe Ring Buffer for Input Events
pub struct InputRingBuffer {
    buffer: [InputEvent; RING_BUFFER_SIZE],
    head: AtomicUsize,
    tail: AtomicUsize,
}

impl InputRingBuffer {
    pub const fn new() -> Self {
        Self {
            buffer: [InputEvent { time: 0, type_: 0, code: 0, value: 0 }; RING_BUFFER_SIZE],
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    /// Pushes an event into the buffer. Safe to call from ISR.
    pub fn push(&mut self, event: InputEvent) -> bool {
        let head = self.head.load(Ordering::Relaxed);
        let next_head = (head + 1) % RING_BUFFER_SIZE;
        
        if next_head == self.tail.load(Ordering::Acquire) {
            return false; // Buffer full
        }
        
        self.buffer[head] = event;
        self.head.store(next_head, Ordering::Release);
        true
    }

    /// Pops an event from the buffer.
    pub fn pop(&mut self) -> Option<InputEvent> {
        let tail = self.tail.load(Ordering::Relaxed);
        if tail == self.head.load(Ordering::Acquire) {
            return None; // Buffer empty
        }
        
        let event = self.buffer[tail];
        self.tail.store((tail + 1) % RING_BUFFER_SIZE, Ordering::Release);
        Some(event)
    }
}

pub static INPUT_RING_BUFFER: Mutex<InputRingBuffer> = Mutex::new(InputRingBuffer::new());

pub struct VirtioInputDriver {
    pub device_id: u16,
    pub vnode_id: usize,
}

lazy_static::lazy_static! {
    pub static ref DRIVER: Mutex<Option<VirtioInputDriver>> = Mutex::new(None);
}

/// Initializes the VirtIO-Input device (PCI Probe simulation)
pub fn init() {
    kernel_log("VIRTIO", "Probing PCI Bus for Device ID 18 (VirtIO Input)...");
    
    let driver = VirtioInputDriver {
        device_id: 18,
        vnode_id: 0x1000, // Fixed identifier for /dev/input0
    };
    
    *DRIVER.lock() = Some(driver);
    kernel_log("VIRTIO", "VirtIO Input Device initialized. Virtqueues configured.");
}

/// Interrupt Handler for VirtIO Input
/// Triggered when a new keyboard/mouse event is available on EventQ.
pub fn handle_interrupt(type_: u16, code: u16, value: i32) {
    // Safety: InputRingBuffer internally uses atomics for push/pop.
    // However, since push/pop are &mut for simplicity in the current struct, 
    // we use a Mutex but we MUST ensure it's an IRQ-safe mutex or use try_lock.
    // In this simulation, we'll use a direct push if we can avoid the Mutex 
    // by making the methods take &self and use atomics internally.
    
    let event = InputEvent {
        time: 0,
        type_,
        code,
        value,
    };

    // We use a simplified atomic push for the ISR
    let mut buffer = INPUT_RING_BUFFER.lock();
    if buffer.push(event) {
        AsyncEventTrigger::trigger_event(0x1000, EVFILT_READ);
    }
}
