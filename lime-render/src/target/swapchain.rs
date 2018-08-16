use std::cell::Cell;
use std::sync::Arc;

use failure;
use vulkano::device::Queue;
use vulkano::format::{D16Unorm, FormatDesc};
use vulkano::framebuffer::{FramebufferAbstract, RenderPassAbstract};
use vulkano::image::AttachmentImage;
use vulkano::swapchain::{
    self, AcquireError, PresentMode, Surface, SurfaceTransform, Swapchain, SwapchainAcquireFuture,
};
use vulkano::sync::GpuFuture;
use winit::Window;

use target::{create_framebuffers, Target};

pub(crate) struct SwapchainTarget {
    swapchain: Arc<Swapchain<Window>>,
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
    framebuffers: Vec<Arc<FramebufferAbstract + Send + Sync>>,
    index: Cell<Option<usize>>,
}

impl SwapchainTarget {
    pub(crate) fn new(
        queue: &Arc<Queue>,
        surface: &Arc<Surface<Window>>,
        render_pass: &Arc<RenderPassAbstract + Send + Sync>,
        dimensions: [u32; 2],
        format: impl FormatDesc,
    ) -> Result<Self, failure::Error> {
        let caps = surface.capabilities(queue.device().physical_device())?;
        let alpha = caps
            .supported_composite_alpha
            .iter()
            .next()
            .expect("surface has no supported alpha modes");

        let (swapchain, images) = Swapchain::new(
            Arc::clone(queue.device()),
            Arc::clone(surface),
            caps.min_image_count,
            format,
            dimensions,
            1,
            caps.supported_usage_flags,
            queue,
            SurfaceTransform::Identity,
            alpha,
            PresentMode::Mailbox,
            true,
            None,
        )?;

        let dbuf = AttachmentImage::transient(Arc::clone(queue.device()), dimensions, D16Unorm)?;
        let framebuffers = create_framebuffers(&render_pass, images, &dbuf)?;

        Ok(SwapchainTarget {
            swapchain,
            render_pass: Arc::clone(render_pass),
            framebuffers,
            index: Cell::default(),
        })
    }
}

impl Target for SwapchainTarget {
    type AcquireFuture = SwapchainAcquireFuture<Window>;

    fn resize(&mut self, queue: &Arc<Queue>, dimensions: [u32; 2]) -> Result<(), failure::Error> {
        let (swapchain, images) = self.swapchain.recreate_with_dimension(dimensions)?;
        self.swapchain = swapchain;
        let dbuf = AttachmentImage::transient(Arc::clone(queue.device()), dimensions, D16Unorm)?;
        self.framebuffers = create_framebuffers(&self.render_pass, images, &dbuf)?;
        Ok(())
    }

    fn acquire(
        &self,
        _: &Arc<Queue>,
    ) -> Result<(Arc<FramebufferAbstract + Send + Sync>, Self::AcquireFuture), AcquireError> {
        let (index, acquire) = swapchain::acquire_next_image(self.swapchain.clone(), None)?;
        self.index.set(Some(index));
        Ok((Arc::clone(&self.framebuffers[index]), acquire))
    }

    fn present<F>(&self, queue: &Arc<Queue>, fut: F) -> Box<GpuFuture + Send + Sync>
    where
        F: GpuFuture + Send + Sync + 'static,
    {
        Box::new(fut.then_swapchain_present(
            Arc::clone(&queue),
            Arc::clone(&self.swapchain),
            self.index.take().expect("swapchain image not acquired"),
        ))
    }
}
