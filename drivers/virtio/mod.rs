pub mod input;
pub mod gpu;

pub fn init() {
    input::init();
    gpu::virtio_gpu_init();
}
