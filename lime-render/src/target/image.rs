use std::sync::Arc;

use failure::Fallible;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::format::{D16Unorm, R8G8B8A8Unorm};
use vulkano::framebuffer::{FramebufferAbstract, RenderPassAbstract};
use vulkano::image::{AttachmentImage, Dimensions, ImageUsage, StorageImage};
use vulkano::instance::{DeviceExtensions, PhysicalDevice};
use vulkano::sync::{now, FenceSignalFuture, GpuFuture};

use target::{create_framebuffer, create_render_pass, Target};
use Context;

pub struct ImageTarget {
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
    framebuffer: Arc<FramebufferAbstract + Send + Sync>,
    image: Arc<StorageImage<R8G8B8A8Unorm>>,
    buffer: Arc<CpuAccessibleBuffer<[u8]>>,
    fence: Option<Arc<FenceSignalFuture<Box<GpuFuture + Send + Sync>>>>,
}

impl ImageTarget {
    pub fn resize(&mut self, ctx: &Context, dimensions: [u32; 2]) -> Fallible<()> {
        let (image, buffer) = create(ctx, dimensions)?;
        let dbuf = AttachmentImage::transient(Arc::clone(ctx.device()), dimensions, D16Unorm)?;
        self.framebuffer =
            create_framebuffer(Arc::clone(&self.render_pass), Arc::clone(&image), dbuf)?;
        self.image = image;
        self.buffer = buffer;
        Ok(())
    }

    pub fn read<R, T>(&mut self, read: R) -> Fallible<T>
    where
        R: FnOnce(&[u8], [u32; 2]) -> Fallible<T>,
    {
        if let Some(fence) = self.fence.take() {
            fence.wait(None)?;
        }

        read(&self.buffer.read()?, self.dimensions())
    }
}

impl Target for ImageTarget {
    type InitData = [u32; 2];

    fn new(phys: PhysicalDevice, dimensions: Self::InitData) -> Fallible<(Self, Context)> {
        let ctx = Context::new(phys, |_| true, &DeviceExtensions::none())?;
        let render_pass = create_render_pass(Arc::clone(ctx.device()), R8G8B8A8Unorm)?;

        let (image, buffer) = create(&ctx, dimensions)?;
        let dbuf = AttachmentImage::transient(Arc::clone(ctx.device()), dimensions, D16Unorm)?;
        let framebuffer = create_framebuffer(Arc::clone(&render_pass), Arc::clone(&image), dbuf)?;
        Ok((
            ImageTarget {
                render_pass,
                framebuffer,
                image,
                buffer,
                fence: None,
            },
            ctx,
        ))
    }

    fn render_pass(&self) -> &Arc<RenderPassAbstract + Send + Sync> {
        &self.render_pass
    }

    fn dimensions(&self) -> [u32; 2] {
        [self.framebuffer.width(), self.framebuffer.height()]
    }

    fn hidpi_factor(&self) -> f32 {
        1.0
    }

    fn recreate(&mut self, _: &Context) -> Fallible<()> {
        Ok(())
    }

    fn acquire(
        &mut self,
        ctx: &Context,
    ) -> Fallible<(
        Arc<FramebufferAbstract + Send + Sync>,
        Box<GpuFuture + Send + Sync>,
    )> {
        Ok((
            Arc::clone(&self.framebuffer),
            Box::new(now(Arc::clone(ctx.device()))),
        ))
    }

    fn present<F>(&mut self, ctx: &Context, fut: F) -> Fallible<Box<GpuFuture + Send + Sync>>
    where
        F: GpuFuture + Send + Sync + 'static,
    {
        let command_buffer =
            AutoCommandBufferBuilder::new(Arc::clone(ctx.device()), ctx.transfer_queue().family())?
                .copy_image_to_buffer(Arc::clone(&self.image), Arc::clone(&self.buffer))?
                .build()?;

        let fut: Box<GpuFuture + Send + Sync> = Box::new(
            fut.then_signal_semaphore()
                .then_execute(Arc::clone(ctx.transfer_queue()), command_buffer)?,
        );
        let fence = Arc::new(fut.then_signal_fence_and_flush()?);

        self.fence = Some(Arc::clone(&fence));

        Ok(Box::new(fence))
    }
}

fn create(
    ctx: &Context,
    [width, height]: [u32; 2],
) -> Fallible<(
    Arc<StorageImage<R8G8B8A8Unorm>>,
    Arc<CpuAccessibleBuffer<[u8]>>,
)> {
    let image = StorageImage::with_usage(
        Arc::clone(ctx.device()),
        Dimensions::Dim2d { width, height },
        R8G8B8A8Unorm,
        ImageUsage {
            color_attachment: true,
            transfer_source: true,
            ..ImageUsage::none()
        },
        ctx.queue_families(),
    )?;

    let buf = CpuAccessibleBuffer::from_iter(
        Arc::clone(ctx.device()),
        BufferUsage::transfer_destination(),
        (0..4 * width * height).map(|_| 0),
    )?;

    Ok((image, buf))
}
