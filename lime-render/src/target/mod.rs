mod image;
mod swapchain;

pub use self::image::ImageTarget;
pub use self::swapchain::SwapchainTarget;

use std::sync::Arc;

use failure::Fallible;
use vulkano::device::Device;
use vulkano::format::{D16Unorm, FormatDesc};
use vulkano::framebuffer::{
    Framebuffer, FramebufferAbstract, FramebufferCreationError, RenderPassAbstract,
};
use vulkano::image::{AttachmentImage, ImageViewAccess};
use vulkano::instance::PhysicalDevice;
use vulkano::sync::GpuFuture;

use Context;

pub trait Target: Sized + Send + Sync + 'static {
    type InitData;

    fn new(phys: PhysicalDevice, Self::InitData) -> Fallible<(Self, Context)>;

    fn render_pass(&self) -> &Arc<RenderPassAbstract + Send + Sync>;
    fn dimensions(&self) -> [u32; 2];
    fn hidpi_factor(&self) -> f32;

    fn logical_size(&self) -> [f32; 2] {
        let [w, h] = self.dimensions();
        let f = self.hidpi_factor();
        [w as f32 / f, h as f32 / f]
    }

    fn recreate(&mut self, ctx: &Context) -> Fallible<()>;

    fn acquire(
        &mut self,
        ctx: &Context,
    ) -> Fallible<(
        Arc<FramebufferAbstract + Send + Sync>,
        Box<GpuFuture + Send + Sync>,
    )>;
    fn present<F>(&mut self, ctx: &Context, fut: F) -> Fallible<Box<GpuFuture + Send + Sync>>
    where
        F: GpuFuture + Send + Sync + 'static;
}

fn create_render_pass(
    device: Arc<Device>,
    format: impl FormatDesc,
) -> Fallible<Arc<RenderPassAbstract + Send + Sync>> {
    Ok(Arc::new(ordered_passes_renderpass!(device,
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: format.format(),
                    samples: 1,
                },
                depth: {
                    load: Clear,
                    store: DontCare,
                    format: D16Unorm.format(),
                    samples: 1,
                }
            },
            passes: [
                {
                    color: [color],
                    depth_stencil: {depth},
                    input: []
                },
                {
                    color: [color],
                    depth_stencil: { },
                    input: []
                }
            ]
    )?))
}

fn create_framebuffers<I: ImageViewAccess + Send + Sync + 'static>(
    pass: &Arc<RenderPassAbstract + Send + Sync>,
    images: impl IntoIterator<Item = Arc<I>>,
    dbuf: &Arc<AttachmentImage<D16Unorm>>,
) -> Result<Vec<Arc<FramebufferAbstract + Send + Sync>>, FramebufferCreationError> {
    images
        .into_iter()
        .map(|img| create_framebuffer(Arc::clone(pass), img, Arc::clone(dbuf)))
        .collect()
}

fn create_framebuffer<I: ImageViewAccess + Send + Sync + 'static>(
    pass: Arc<RenderPassAbstract + Send + Sync>,
    img: Arc<I>,
    dbuf: Arc<AttachmentImage<D16Unorm>>,
) -> Result<Arc<FramebufferAbstract + Send + Sync>, FramebufferCreationError> {
    Ok(Arc::new(
        Framebuffer::start(pass).add(img)?.add(dbuf)?.build()?,
    ))
}
