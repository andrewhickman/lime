use std::sync::Arc;

use failure::Fallible;
use vulkano::format::D16Unorm;
use vulkano::framebuffer::{FramebufferAbstract, RenderPassAbstract};
use vulkano::image::AttachmentImage;
use vulkano::instance::{DeviceExtensions, PhysicalDevice};
use vulkano::swapchain::{self, PresentMode, Surface, SurfaceTransform, Swapchain};
use vulkano::sync::GpuFuture;
use vulkano_win;
use winit::Window;

use target::{create_framebuffers, create_render_pass, Target};
use Context;

pub struct SwapchainTarget {
    surface: Arc<Surface<Window>>,
    swapchain: Arc<Swapchain<Window>>,
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
    framebuffers: Vec<Arc<FramebufferAbstract + Send + Sync>>,
    index: Option<usize>,
    dimensions: [u32; 2],
}

impl SwapchainTarget {
    pub fn window(&self) -> &Window {
        self.surface.window()
    }
}

impl Target for SwapchainTarget {
    type InitData = Window;

    fn new(phys: PhysicalDevice, window: Self::InitData) -> Fallible<(Self, Context)> {
        let surface = vulkano_win::create_vk_surface(window, Arc::clone(phys.instance()))?;

        let ctx = Context::new(
            phys,
            |fam| surface.is_supported(fam).unwrap_or(false),
            &DeviceExtensions {
                khr_swapchain: true,
                ..DeviceExtensions::none()
            },
        )?;

        let dpi_factor = surface.window().get_hidpi_factor();
        let logical_size = surface.window().get_inner_size().unwrap();
        let (w, h) = logical_size.to_physical(dpi_factor).into();

        let caps = surface.capabilities(ctx.device().physical_device())?;
        let &(format, _) = caps
            .supported_formats
            .first()
            .expect("surface has no supported formats");

        let render_pass = create_render_pass(Arc::clone(ctx.device()), format)?;

        let caps = surface.capabilities(phys)?;
        let alpha = caps
            .supported_composite_alpha
            .iter()
            .next()
            .expect("surface has no supported alpha modes");

        let (swapchain, images) = Swapchain::new(
            Arc::clone(ctx.device()),
            Arc::clone(&surface),
            caps.min_image_count,
            format,
            [w, h],
            1,
            caps.supported_usage_flags,
            ctx.graphics_queue(),
            SurfaceTransform::Identity,
            alpha,
            PresentMode::Mailbox,
            true,
            None,
        )?;

        let dbuf = AttachmentImage::transient(Arc::clone(ctx.device()), [w, h], D16Unorm)?;
        let framebuffers = create_framebuffers(&render_pass, images, &dbuf)?;

        Ok((
            SwapchainTarget {
                surface,
                swapchain,
                render_pass,
                framebuffers,
                index: None,
                dimensions: [w, h],
            },
            ctx,
        ))
    }

    fn render_pass(&self) -> &Arc<RenderPassAbstract + Send + Sync> {
        &self.render_pass
    }

    fn dimensions(&self) -> [u32; 2] {
        self.dimensions
    }

    fn hidpi_factor(&self) -> f32 {
        self.surface.window().get_hidpi_factor() as f32
    }

    fn recreate(&mut self, ctx: &Context) -> Fallible<()> {
        self.dimensions = self
            .surface
            .capabilities(ctx.device().physical_device())?
            .current_extent
            .unwrap();

        let (swapchain, images) = self.swapchain.recreate_with_dimension(self.dimensions)?;
        self.swapchain = swapchain;
        let dbuf = AttachmentImage::transient(Arc::clone(ctx.device()), self.dimensions, D16Unorm)?;
        self.framebuffers = create_framebuffers(&self.render_pass, images, &dbuf)?;
        Ok(())
    }

    fn acquire(
        &mut self,
        _: &Context,
    ) -> Fallible<(
        Arc<FramebufferAbstract + Send + Sync>,
        Box<GpuFuture + Send + Sync>,
    )> {
        let (index, acquire) = swapchain::acquire_next_image(self.swapchain.clone(), None)?;
        self.index = Some(index);
        Ok((Arc::clone(&self.framebuffers[index]), Box::new(acquire)))
    }

    fn present<F>(&mut self, ctx: &Context, fut: F) -> Fallible<Box<GpuFuture + Send + Sync>>
    where
        F: GpuFuture + Send + Sync + 'static,
    {
        Ok(Box::new(fut.then_swapchain_present(
            Arc::clone(ctx.graphics_queue()),
            Arc::clone(&self.swapchain),
            self.index.take().expect("swapchain image not acquired"),
        )))
    }
}
