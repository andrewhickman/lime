mod swapchain;

pub(crate) use self::swapchain::SwapchainTarget;

use std::sync::Arc;

use failure;
use vulkano::device::Queue;
use vulkano::format::D16Unorm;
use vulkano::framebuffer::{
    Framebuffer, FramebufferAbstract, FramebufferCreationError, RenderPassAbstract,
};
use vulkano::image::{AttachmentImage, ImageViewAccess};
use vulkano::swapchain::AcquireError;
use vulkano::sync::GpuFuture;

pub(crate) trait Target {
    type AcquireFuture: GpuFuture + Send + Sync + 'static;

    fn resize(&mut self, queue: &Arc<Queue>, dimensions: [u32; 2]) -> Result<(), failure::Error>;
    fn acquire(
        &self,
        queue: &Arc<Queue>,
    ) -> Result<(Arc<FramebufferAbstract + Send + Sync>, Self::AcquireFuture), AcquireError>;
    fn present<F>(&self, queue: &Arc<Queue>, fut: F) -> Box<GpuFuture + Send + Sync>
    where
        F: GpuFuture + Send + Sync + 'static;
}

fn create_framebuffers<I: ImageViewAccess + Send + Sync + 'static>(
    pass: &Arc<RenderPassAbstract + Send + Sync>,
    images: impl IntoIterator<Item = Arc<I>>,
    dbuf: &Arc<AttachmentImage<D16Unorm>>,
) -> Result<Vec<Arc<FramebufferAbstract + Send + Sync>>, FramebufferCreationError> {
    images
        .into_iter()
        .map(|img| create_framebuffer(pass, img, dbuf))
        .collect()
}

fn create_framebuffer<I: ImageViewAccess + Send + Sync + 'static>(
    pass: &Arc<RenderPassAbstract + Send + Sync>,
    img: Arc<I>,
    dbuf: &Arc<AttachmentImage<D16Unorm>>,
) -> Result<Arc<FramebufferAbstract + Send + Sync>, FramebufferCreationError> {
    Ok(Arc::new(
        Framebuffer::start(Arc::clone(pass))
            .add(img)?
            .add(Arc::clone(dbuf))?
            .build()?,
    ))
}
