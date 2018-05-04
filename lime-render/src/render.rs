use std::iter;
use std::sync::Arc;

use specs::shred::Resources;
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::format::{D16Unorm, Format};
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass};
use vulkano::image::{AttachmentImage, SwapchainImage};
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::pipeline::viewport::Viewport;
use vulkano::swapchain::{self, AcquireError, PresentMode, Surface, SurfaceTransform, Swapchain,
                         SwapchainCreationError};
use vulkano::sync::{FlushError, GpuFuture};
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
            Arc::clone(&queue.device()),
            Subpass::from(Arc::clone(&render_pass), 0).unwrap(),
        );
        let d2 = d2::Renderer::new(
            Arc::clone(&queue.device()),
            Subpass::from(Arc::clone(&render_pass), 1).unwrap(),
        );

        let depth_buffer = AttachmentImage::transient(Arc::clone(queue.device()), [w, h], D16Unorm)
            .unwrap_or_else(quit);
        let framebuffers = create_framebuffers(Arc::clone(&render_pass), images, depth_buffer);

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
                if self.recreate_swapchain(dim) {
                    trace!("Recreate swapchain succeeded.");
                    swapchain_dirty = false;
                } else {
                    trace!("Recreate swapchain failed.");
                    break;
                }
            } else {
                if self.try_render(res, d3, d2, dim) {
                    trace!("Draw succeeded.");
                    break;
                } else {
                    trace!("Draw failed.");
                    swapchain_dirty = true;
                }
            }
        }
    }

    fn recreate_swapchain(&mut self, dim: &mut ScreenDimensions) -> bool {
        let new_dim = self.new_dimensions();
        match self.swapchain.recreate_with_dimension(new_dim.into()) {
            Ok((swapchain, images)) => {
                self.swapchain = swapchain;
                let depth_buffer = AttachmentImage::transient(
                    Arc::clone(self.queue.device()),
                    new_dim.into(),
                    D16Unorm,
                ).unwrap_or_else(quit);
                self.framebuffers =
                    create_framebuffers(Arc::clone(&self.render_pass), images, depth_buffer);
                *dim = new_dim;
                true
            }
            Err(SwapchainCreationError::UnsupportedDimensions) => false,
            Err(err) => quit(err),
        }
    }

    fn try_render<D3: d3::Draw, D2: d2::Draw>(
        &mut self,
        res: &Resources,
        d3: &D3,
        d2: &D2,
        dim: &mut ScreenDimensions,
    ) -> bool {
        let (image_num, acquire) = match swapchain::acquire_next_image(self.swapchain.clone(), None)
        {
            Ok(r) => r,
            Err(AcquireError::OutOfDate) => return false,
            Err(err) => quit(err),
        };

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
        ).unwrap_or_else(quit)
            .begin_render_pass(fb, false, vec![[0.0, 0.0, 0.0, 1.0].into(), 1f32.into()])
            .unwrap_or_else(quit);
        let command_buffer = self.d3
            .draw(res, command_buffer, d3, state.clone())
            .next_subpass(false)
            .unwrap_or_else(quit);
        let command_buffer = self.d2
            .draw(res, command_buffer, d2, state)
            .end_render_pass()
            .unwrap_or_else(quit)
            .build()
            .unwrap_or_else(quit);

        let future = match self.last_frame.take() {
            Some(last_frame) => last_frame
                .join(acquire)
                .then_execute(Arc::clone(&self.queue), command_buffer)
                .unwrap_or_else(quit)
                .then_swapchain_present(
                    Arc::clone(&self.queue),
                    Arc::clone(&self.swapchain),
                    image_num,
                )
                .then_signal_fence_and_flush()
                .map(|f| Box::new(f) as Box<GpuFuture + Send + Sync>),
            None => acquire
                .then_execute(Arc::clone(&self.queue), command_buffer)
                .unwrap_or_else(quit)
                .then_swapchain_present(
                    Arc::clone(&self.queue),
                    Arc::clone(&self.swapchain),
                    image_num,
                )
                .then_signal_fence_and_flush()
                .map(|f| Box::new(f) as Box<GpuFuture + Send + Sync>),
        };

        match future {
            Ok(future) => self.last_frame = Some(future),
            Err(FlushError::OutOfDate) => return false,
            Err(err) => quit(err),
        }

        true
    }
}

fn create_framebuffers<I>(
    pass: Arc<RenderPassAbstract + Send + Sync>,
    images: I,
    dbuf: Arc<AttachmentImage<D16Unorm>>,
) -> Vec<Arc<FramebufferAbstract + Send + Sync>>
where
    I: IntoIterator<Item = Arc<SwapchainImage<Window>>>,
{
    images
        .into_iter()
        .map(|img| create_framebuffer(Arc::clone(&pass), img, Arc::clone(&dbuf)))
        .collect()
}

fn create_framebuffer(
    pass: Arc<RenderPassAbstract + Send + Sync>,
    img: Arc<SwapchainImage<Window>>,
    dbuf: Arc<AttachmentImage<D16Unorm>>,
) -> Arc<FramebufferAbstract + Send + Sync> {
    Arc::new(
        Framebuffer::start(pass)
            .add(img)
            .unwrap_or_else(quit)
            .add(dbuf)
            .unwrap_or_else(quit)
            .build()
            .unwrap_or_else(quit),
    )
}
