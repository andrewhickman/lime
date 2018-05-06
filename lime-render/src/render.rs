use std::iter;
use std::sync::Arc;

use failure;
use shrev::EventChannel;
use specs::shred::Resources;
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::format::{D16Unorm, Format};
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass};
use vulkano::image::{AttachmentImage, SwapchainImage};
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::pipeline::viewport::Viewport;
use vulkano::swapchain::{self, PresentMode, Surface, SurfaceTransform, Swapchain};
use vulkano::sync::GpuFuture;
use vulkano_win::{self, VkSurfaceBuild};
use winit::{EventsLoop, Window, WindowBuilder};

use {quit, quit_msg, ScreenDimensions, d2, d3};

pub struct Renderer {
    pub(crate) d2: d2::Renderer,
    pub(crate) d3: d3::Renderer,
    pub(crate) queue: Arc<Queue>,
    surface: Arc<Surface<Window>>,
    swapchain: Arc<Swapchain<Window>>,
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
    last_frame: Option<Box<GpuFuture + Send + Sync>>,
    framebuffers: Vec<Arc<FramebufferAbstract + Send + Sync>>,
}

impl Renderer {
    pub fn new(events_loop: &EventsLoop, builder: WindowBuilder) -> Self {
        let instance = {
            let extensions = vulkano_win::required_extensions();
            Instance::new(None, &extensions, None).unwrap_or_else(quit)
        };

        let phys = PhysicalDevice::enumerate(&instance)
            .next()
            .unwrap_or_else(|| quit_msg("no device available"));
        info!("Using device: {} (type: {:?}).", phys.name(), phys.ty());

        let surface = builder
            .build_vk_surface(&events_loop, Arc::clone(&instance))
            .unwrap_or_else(quit);

        let queue_family = phys.queue_families()
            .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
            .unwrap_or_else(|| quit_msg("couldn't find a graphical queue family"));

        let (_, mut queues) = {
            let device_ext = DeviceExtensions {
                khr_swapchain: true,
                ..DeviceExtensions::none()
            };

            Device::new(
                phys,
                phys.supported_features(),
                &device_ext,
                iter::once((queue_family, 0.5)),
            ).unwrap_or_else(quit)
        };

        let queue = queues.next().unwrap();

        let caps = surface
            .capabilities(queue.device().physical_device())
            .unwrap_or_else(quit);
        let format = caps.supported_formats
            .first()
            .unwrap_or_else(|| quit_msg("surface has no supported formats"));
        let alpha = caps.supported_composite_alpha
            .iter()
            .next()
            .unwrap_or_else(|| quit_msg("surface has no supported alpha modes"));

        let (w, h) = surface.window().get_inner_size().unwrap();
        let (swapchain, images) = Swapchain::new(
            Arc::clone(queue.device()),
            Arc::clone(&surface),
            caps.min_image_count,
            format.0,
            [w, h],
            1,
            caps.supported_usage_flags,
            &queue,
            SurfaceTransform::Identity,
            alpha,
            PresentMode::Mailbox,
            true,
            None,
        ).unwrap_or_else(quit);

        let render_pass = Arc::new(
            ordered_passes_renderpass!(Arc::clone(queue.device()),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.format(),
                    samples: 1,
                },
                depth: {
                    load: Clear,
                    store: DontCare,
                    format: Format::D16Unorm,
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
        ).unwrap_or_else(quit),
        ) as Arc<RenderPassAbstract + Send + Sync>;

        let d3 = d3::Renderer::new(
            queue.device(),
            Subpass::from(Arc::clone(&render_pass), 0).unwrap(),
        );
        let d2 = d2::Renderer::new(
            queue.device(),
            Subpass::from(Arc::clone(&render_pass), 1).unwrap(),
        );

        let depth_buffer = AttachmentImage::transient(Arc::clone(queue.device()), [w, h], D16Unorm)
            .unwrap_or_else(quit);
        let framebuffers = create_framebuffers(&render_pass, images, &depth_buffer)
            .unwrap_or_else(quit);

        Renderer {
            surface,
            queue,
            swapchain,
            framebuffers,
            render_pass,
            last_frame: None,
            d2,
            d3,
        }
    }

    pub(crate) fn dimensions(&self) -> ScreenDimensions {
        self.swapchain.dimensions().into()
    }

    fn new_dimensions(&self) -> ScreenDimensions {
        self.surface
            .capabilities(self.queue.device().physical_device())
            .unwrap_or_else(quit)
            .current_extent
            .unwrap()
            .into()
    }

    pub(crate) fn render<D3: d3::Draw, D2: d2::Draw>(
        &mut self,
        res: &Resources,
        d3: &D3,
        d2: &D2,
        dim: &mut ScreenDimensions,
    ) {
        if let Some(ref mut last_frame) = self.last_frame {
            last_frame.cleanup_finished();
        }

        let mut swapchain_dirty = false;
        for _ in 0..5 {
            if swapchain_dirty {
                match self.recreate_swapchain(res, dim) {
                    Ok(()) => {
                        trace!("Recreate swapchain succeeded.");
                        swapchain_dirty = false;
                    }
                    Err(err) => {
                        trace!("Recreate swapchain failed: {}.", err);
                        break;
                    }
                }
            } else {
                match self.try_render(res, dim, d3, d2) {
                    Ok(()) => {
                        trace!("Draw succeeded.");
                        break;
                    }
                    Err(err) => {
                        trace!("Draw failed: {}.", err);
                        swapchain_dirty = true;
                    }
                }
            }
        }
    }

    fn recreate_swapchain(
        &mut self,
        res: &Resources,
        dim: &mut ScreenDimensions,
    ) -> Result<(), failure::Error> {
        let new_dim = self.new_dimensions();
        let (swapchain, images) = self.swapchain.recreate_with_dimension(new_dim.into())?;
        self.swapchain = swapchain;
        let depth_buffer =
            AttachmentImage::transient(Arc::clone(self.queue.device()), new_dim.into(), D16Unorm)?;
        self.framebuffers =
            create_framebuffers(&self.render_pass, images, &depth_buffer)?;
        *dim = new_dim;
        res.fetch_mut::<EventChannel<ScreenDimensions>>()
            .single_write(new_dim);
        Ok(())
    }

    fn try_render<D3: d3::Draw, D2: d2::Draw>(
        &mut self,
        res: &Resources,
        dim: &mut ScreenDimensions,
        d3: &D3,
        d2: &D2,
    ) -> Result<(), failure::Error> {
        let (image_num, acquire) = swapchain::acquire_next_image(self.swapchain.clone(), None)?;
        let fb = Arc::clone(&self.framebuffers[image_num]);

        let state = DynamicState {
            line_width: None,
            viewports: Some(vec![
                Viewport {
                    origin: [0.0, 0.0],
                    dimensions: [dim.w as f32, dim.h as f32],
                    depth_range: 0.0..1.0,
                },
            ]),
            scissors: None,
        };

        let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(
            Arc::clone(self.queue.device()),
            self.queue.family(),
        )?.begin_render_pass(
            fb,
            false,
            vec![[0.0, 0.0, 0.0, 1.0].into(), 1f32.into()],
        )?;
        let command_buffer = self.d3
            .draw(res, command_buffer, d3, state.clone())?
            .next_subpass(false)?;
        let command_buffer = self.d2
            .draw(res, command_buffer, d2, state)?
            .end_render_pass()?
            .build()?;

        let future = match self.last_frame.take() {
            Some(last_frame) => last_frame
                .join(acquire)
                .then_execute(Arc::clone(&self.queue), command_buffer)?
                .then_swapchain_present(
                    Arc::clone(&self.queue),
                    Arc::clone(&self.swapchain),
                    image_num,
                )
                .then_signal_fence_and_flush()
                .map(|f| Box::new(f) as Box<GpuFuture + Send + Sync>),
            None => acquire
                .then_execute(Arc::clone(&self.queue), command_buffer)?
                .then_swapchain_present(
                    Arc::clone(&self.queue),
                    Arc::clone(&self.swapchain),
                    image_num,
                )
                .then_signal_fence_and_flush()
                .map(|f| Box::new(f) as Box<GpuFuture + Send + Sync>),
        };

        self.last_frame = Some(future?);
        Ok(())
    }
}

fn create_framebuffers<I>(
    pass: &Arc<RenderPassAbstract + Send + Sync>,
    images: I,
    dbuf: &Arc<AttachmentImage<D16Unorm>>,
) -> Result<Vec<Arc<FramebufferAbstract + Send + Sync>>, failure::Error>
where
    I: IntoIterator<Item = Arc<SwapchainImage<Window>>>,
{
    images
        .into_iter()
        .map(|img| create_framebuffer(pass, img, dbuf))
        .collect()
}

fn create_framebuffer(
    pass: &Arc<RenderPassAbstract + Send + Sync>,
    img: Arc<SwapchainImage<Window>>,
    dbuf: &Arc<AttachmentImage<D16Unorm>>,
) -> Result<Arc<FramebufferAbstract + Send + Sync>, failure::Error> {
    Ok(Arc::new(Framebuffer::start(Arc::clone(pass))
        .add(img)?
        .add(Arc::clone(dbuf))?
        .build()?))
}
