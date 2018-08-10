use std::cell::Cell;
use std::sync::Arc;

use failure;
use vulkano::device::Queue;
use vulkano::format::{D16Unorm, Format};
use vulkano::framebuffer::{
    Framebuffer, FramebufferAbstract, FramebufferCreationError, RenderPassAbstract,
};
use vulkano::image::{AttachmentImage, SwapchainImage};
use vulkano::swapchain::{
    self, AcquireError, PresentMode, Surface, SurfaceTransform, Swapchain, SwapchainAcquireFuture,
};
use vulkano::sync::GpuFuture;
use winit::Window;

use target::Target;

pub(crate) struct SwapchainTarget {
    swapchain: Arc<Swapchain<Window>>,
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
    framebuffers: Vec<Arc<FramebufferAbstract + Send + Sync>>,
    index: Cell<Option<usize>>,
}

impl SwapchainTarget {
    pub(crate) fn new<F>(
        queue: &Arc<Queue>,
        surface: &Arc<Surface<Window>>,
        dimensions: [u32; 2],
        render_pass: F,
    ) -> Result<Self, failure::Error>
    where
        F: FnOnce(Format) -> Arc<RenderPassAbstract + Send + Sync>,
    {
        let caps = surface.capabilities(queue.device().physical_device())?;
        let format = caps
            .supported_formats
            .first()
            .expect("surface has no supported formats");
        let alpha = caps
            .supported_composite_alpha
            .iter()
            .next()
            .expect("surface has no supported alpha modes");

        let (swapchain, images) = Swapchain::new(
            Arc::clone(queue.device()),
            Arc::clone(surface),
            caps.min_image_count,
            format.0,
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

        let render_pass = render_pass(swapchain.format());
        let dbuf = AttachmentImage::transient(Arc::clone(queue.device()), dimensions, D16Unorm)?;
        let framebuffers = create_framebuffers(&render_pass, images, &dbuf)?;

        Ok(SwapchainTarget {
            swapchain,
            render_pass,
            framebuffers,
            index: Cell::default(),
        })
    }
}

impl Target for SwapchainTarget {
    type AcquireFuture = SwapchainAcquireFuture<Window>;

    fn render_pass(&self) -> Arc<RenderPassAbstract + Send + Sync> {
        Arc::clone(&self.render_pass)
    }

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

fn create_framebuffers(
    pass: &Arc<RenderPassAbstract + Send + Sync>,
    images: impl IntoIterator<Item = Arc<SwapchainImage<Window>>>,
    dbuf: &Arc<AttachmentImage<D16Unorm>>,
) -> Result<Vec<Arc<FramebufferAbstract + Send + Sync>>, FramebufferCreationError> {
    images
        .into_iter()
        .map(|img| create_framebuffer(pass, img, dbuf))
        .collect()
}

fn create_framebuffer(
    pass: &Arc<RenderPassAbstract + Send + Sync>,
    img: Arc<SwapchainImage<Window>>,
    dbuf: &Arc<AttachmentImage<D16Unorm>>,
) -> Result<Arc<FramebufferAbstract + Send + Sync>, FramebufferCreationError> {
    Ok(Arc::new(
        Framebuffer::start(Arc::clone(pass))
            .add(img)?
            .add(Arc::clone(dbuf))?
            .build()?,
    ))
}
