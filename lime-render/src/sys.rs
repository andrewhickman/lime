use std::iter;
use std::sync::Arc;

use failure;
use shrev::{EventChannel, ReaderId};
use specs::prelude::*;
use utils::throw;
use vulkano::command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder, DynamicState};
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::format::{D16Unorm, Format};
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass};
use vulkano::image::{AttachmentImage, SwapchainImage};
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::pipeline::viewport::Viewport;
use vulkano::swapchain::{self, PresentMode, Surface, SurfaceTransform, Swapchain};
use vulkano::sync::GpuFuture;
use vulkano_win;
use winit::{self, Window, WindowEvent};

use {d2, d3};

pub struct RenderSystem {
    pub(crate) queue: Arc<Queue>,
    surface: Arc<Surface<Window>>,
    swapchain: Arc<Swapchain<Window>>,
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
    prev_frame: Option<Box<GpuFuture + Send + Sync>>,
    framebuffers: Vec<Arc<FramebufferAbstract + Send + Sync>>,
    swapchain_dirty: bool,
    dpi_factor: f32,
    event_rx: ReaderId<winit::Event>,
    dimensions: [f32; 2],
}

impl RenderSystem {
    pub const NAME: &'static str = "render::Render";

    pub(crate) fn add(
        world: &mut World,
        dispatcher: &mut DispatcherBuilder,
        window: Window,
        d3_sys: &str,
        d2_sys: &str,
    ) {
        let instance = {
            let extensions = vulkano_win::required_extensions();
            Instance::new(None, &extensions, None).unwrap_or_else(throw)
        };

        let phys = PhysicalDevice::enumerate(&instance)
            .next()
            .expect("no device available");
        info!("Using device: {} (type: {:?}).", phys.name(), phys.ty());

        let surface =
            vulkano_win::create_vk_surface(window, Arc::clone(&instance)).unwrap_or_else(throw);

        let queue_family = phys.queue_families()
            .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
            .expect("couldn't find a graphical queue family");

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
            ).unwrap_or_else(throw)
        };

        let queue = queues.next().unwrap();

        let caps = surface
            .capabilities(queue.device().physical_device())
            .unwrap_or_else(throw);
        let format = caps.supported_formats
            .first()
            .expect("surface has no supported formats");
        let alpha = caps.supported_composite_alpha
            .iter()
            .next()
            .expect("surface has no supported alpha modes");

        let dpi_factor = surface.window().get_hidpi_factor();
        let logical_size = surface.window().get_inner_size().unwrap();
        let (w, h) = logical_size.to_physical(dpi_factor).into();
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
        ).unwrap_or_else(throw);

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
        ).unwrap_or_else(throw),
        ) as Arc<RenderPassAbstract + Send + Sync>;

        let depth_buffer = AttachmentImage::transient(Arc::clone(queue.device()), [w, h], D16Unorm)
            .unwrap_or_else(throw);
        let framebuffers =
            create_framebuffers(&render_pass, images, &depth_buffer).unwrap_or_else(throw);

        let event_rx = {
            let mut event_tx = world.write_resource::<EventChannel<winit::Event>>();
            let event_rx = event_tx.register_reader();
            event_tx.single_write(winit::Event::WindowEvent {
                event: WindowEvent::Resized(logical_size),
                window_id: surface.window().id(),
            });
            event_rx
        };

        world.add_resource(d3::Renderer::new(
            queue.device(),
            Subpass::from(Arc::clone(&render_pass), 0).unwrap(),
        ));
        world.add_resource(d2::Renderer::new(
            queue.device(),
            Subpass::from(Arc::clone(&render_pass), 1).unwrap(),
        ));

        dispatcher.add(
            RenderSystem {
                surface,
                queue,
                swapchain,
                framebuffers,
                render_pass,
                prev_frame: None,
                swapchain_dirty: false,
                dpi_factor: dpi_factor as f32,
                event_rx,
                dimensions: [w as f32, h as f32],
            },
            RenderSystem::NAME,
            &[d3_sys, d2_sys],
        )
    }

    fn render(&mut self, d3: &mut d3::Renderer, d2: &mut d2::Renderer) {
        for _ in 0..5 {
            if self.swapchain_dirty {
                match self.recreate_swapchain() {
                    Ok(()) => {
                        trace!("Recreate swapchain succeeded");
                        self.swapchain_dirty = false;
                    }
                    Err(err) => {
                        trace!("Recreate swapchain failed: {}.", err);
                        break;
                    }
                }
            } else {
                match self.try_render(d3, d2) {
                    Ok(()) => {
                        trace!("Draw succeeded.");
                        break;
                    }
                    Err(err) => {
                        trace!("Draw failed: {}.", err);
                        self.swapchain_dirty = true;
                    }
                }
            }
        }
    }

    fn recreate_swapchain(&mut self) -> Result<(), failure::Error> {
        let dimensions = self.surface
            .capabilities(self.queue.device().physical_device())?
            .current_extent
            .unwrap();
        let (swapchain, images) = self.swapchain.recreate_with_dimension(dimensions)?;
        self.swapchain = swapchain;
        let depth_buffer =
            AttachmentImage::transient(Arc::clone(self.queue.device()), dimensions, D16Unorm)?;
        self.framebuffers = create_framebuffers(&self.render_pass, images, &depth_buffer)?;
        self.dimensions = [dimensions[0] as f32, dimensions[1] as f32];
        Ok(())
    }

    fn try_render(
        &mut self,
        d3: &mut d3::Renderer,
        d2: &mut d2::Renderer,
    ) -> Result<(), failure::Error> {
        let (img_num, acquire) = swapchain::acquire_next_image(self.swapchain.clone(), None)?;
        let fb = Arc::clone(&self.framebuffers[img_num]);
        if let Some(ref mut last_frame) = self.prev_frame {
            last_frame.cleanup_finished();
        }

        let state = DynamicState {
            line_width: None,
            viewports: Some(vec![Viewport {
                origin: [0.0, 0.0],
                dimensions: self.dimensions,
                depth_range: 0.0..1.0,
            }]),
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
        let command_buffer = d3.draw(command_buffer, state.clone())?.next_subpass(false)?;
        let command_buffer = d2.draw(command_buffer, state, self.logical_size())?
            .end_render_pass()?
            .build()?;

        self.prev_frame = Some(match self.prev_frame.take() {
            Some(last_frame) => self.execute(last_frame.join(acquire), command_buffer, img_num)?,
            None => self.execute(acquire, command_buffer, img_num)?,
        });
        Ok(())
    }

    fn execute(
        &self,
        acquire_future: impl GpuFuture,
        command_buffer: AutoCommandBuffer,
        image_num: usize,
    ) -> Result<Box<impl GpuFuture>, failure::Error> {
        let future = acquire_future
            .then_execute(Arc::clone(&self.queue), command_buffer)?
            .then_signal_fence()
            .then_swapchain_present(
                Arc::clone(&self.queue),
                Arc::clone(&self.swapchain),
                image_num,
            );
        future.flush()?;
        Ok(Box::new(future))
    }

    fn logical_size(&self) -> [f32; 2] {
        let [w, h] = self.dimensions;
        [w / self.dpi_factor, h / self.dpi_factor]
    }
}

fn create_framebuffers(
    pass: &Arc<RenderPassAbstract + Send + Sync>,
    images: impl IntoIterator<Item = Arc<SwapchainImage<Window>>>,
    dbuf: &Arc<AttachmentImage<D16Unorm>>,
) -> Result<Vec<Arc<FramebufferAbstract + Send + Sync>>, failure::Error> {
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

impl<'a> System<'a> for RenderSystem {
    type SystemData = (
        ReadExpect<'a, EventChannel<winit::Event>>,
        WriteExpect<'a, d3::Renderer>,
        WriteExpect<'a, d2::Renderer>,
    );

    fn run(&mut self, (event_tx, mut d3, mut d2): Self::SystemData) {
        for event in event_tx.read(&mut self.event_rx) {
            if let winit::Event::WindowEvent {
                event: WindowEvent::HiDpiFactorChanged(dpi_factor),
                ..
            } = event
            {
                self.dpi_factor = *dpi_factor as f32;
            }
        }

        self.render(&mut d3, &mut d2);
    }
}
