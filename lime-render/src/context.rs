use std::sync::Arc;

use vulkano::device::{Device, Queue};

pub struct Context {
    transfer_queue: Arc<Queue>,
    graphics_queue: Arc<Queue>,
}

impl Context {
    pub fn transfer_queue(&self) -> &Arc<Queue> {
        &self.transfer_queue
    }

    pub fn graphics_queue(&self) -> &Arc<Queue> {
        &self.graphics_queue
    }

    pub fn device(&self) -> &Arc<Device> {
        self.transfer_queue.device()
    }
}
