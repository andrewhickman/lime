mod swapchain;

pub(crate) use self::swapchain::SwapchainTarget;

use std::sync::Arc;

use failure;
use vulkano::device::Queue;
use vulkano::framebuffer::{FramebufferAbstract, RenderPassAbstract};
use vulkano::swapchain::AcquireError;
use vulkano::sync::GpuFuture;

pub(crate) trait Target {
    type AcquireFuture: GpuFuture + Send + Sync + 'static;

    fn render_pass(&self) -> Arc<RenderPassAbstract + Send + Sync>;
    fn resize(&mut self, queue: &Arc<Queue>, dimensions: [u32; 2]) -> Result<(), failure::Error>;
    fn acquire(
        &self,
        queue: &Arc<Queue>,
    ) -> Result<(Arc<FramebufferAbstract + Send + Sync>, Self::AcquireFuture), AcquireError>;
    fn present<F>(&self, queue: &Arc<Queue>, fut: F) -> Box<GpuFuture + Send + Sync>
    where
        F: GpuFuture + Send + Sync + 'static;
}
