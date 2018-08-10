use std::iter;
use std::sync::Arc;

use failure;
use shrev::{EventChannel, ReaderId};
use specs::prelude::*;
use utils::throw;
use vulkano::command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder, DynamicState};
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::format::Format;
use vulkano::framebuffer::Subpass;
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::pipeline::viewport::Viewport;
use vulkano::swapchain::Surface;
use vulkano::sync::GpuFuture;
use vulkano_win;
use winit::{self, Window, WindowEvent};

use target::{SwapchainTarget, Target};
use {d2, d3};

pub(crate) struct RenderSystem<T> {
    pub(crate) queue: Arc<Queue>,
    surface: Arc<Surface<Window>>,
    prev_frame: Option<Box<GpuFuture + Send + Sync>>,
    swapchain_dirty: bool,
    dpi_factor: f32,
    event_rx: ReaderId<winit::Event>,
    dimensions: [f32; 2],
    state: DynamicState,
    target: T,
}

impl RenderSystem<SwapchainTarget> {
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

        let queue_family = phys
            .queue_families()
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

        let dpi_factor = surface.window().get_hidpi_factor();
        let logical_size = surface.window().get_inner_size().unwrap();
        let (w, h) = logical_size.to_physical(dpi_factor).into();

        let target = SwapchainTarget::new(&queue, &surface, [w, h], |format| {
            Arc::new(
                ordered_passes_renderpass!(Arc::clone(queue.device()),
                attachments: {
                    color: {
                        load: Clear,
                        store: Store,
                        format: format,
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
            )
        }).unwrap_or_else(throw);

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
            Subpass::from(target.render_pass(), 0).unwrap(),
        ));
        world.add_resource(d2::Renderer::new(
            queue.device(),
            Subpass::from(target.render_pass(), 1).unwrap(),
        ));

        let dimensions = [w as f32, h as f32];

        let state = DynamicState {
            line_width: None,
            viewports: Some(vec![Viewport {
                origin: [0.0, 0.0],
                dimensions,
                depth_range: 0.0..1.0,
            }]),
            scissors: None,
        };

        dispatcher.add(
            RenderSystem {
                surface,
                queue,
                prev_frame: None,
                swapchain_dirty: false,
                dpi_factor: dpi_factor as f32,
                event_rx,
                target,
                dimensions,
                state,
            },
            RenderSystem::NAME,
            &[d3_sys, d2_sys],
        )
    }
}

impl<T> RenderSystem<T>
where
    T: Target,
{
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
        let dimensions = self
            .surface
            .capabilities(self.queue.device().physical_device())?
            .current_extent
            .unwrap();
        self.target.resize(&self.queue, dimensions)?;
        self.dimensions = [dimensions[0] as f32, dimensions[1] as f32];
        self.state.viewports.as_mut().unwrap()[0].dimensions = self.dimensions;
        Ok(())
    }

    fn try_render(
        &mut self,
        d3: &mut d3::Renderer,
        d2: &mut d2::Renderer,
    ) -> Result<(), failure::Error> {
        let (fb, acquire) = self.target.acquire(&self.queue)?;

        if let Some(ref mut last_frame) = self.prev_frame {
            last_frame.cleanup_finished();
        }

        let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(
            Arc::clone(self.queue.device()),
            self.queue.family(),
        )?.begin_render_pass(
            fb,
            false,
            vec![[0.0, 0.0, 0.0, 1.0].into(), 1f32.into()],
        )?;
        let command_buffer = d3.commit(command_buffer, &self.state)?.next_subpass(false)?;
        let command_buffer = d2
            .commit(command_buffer, &self.state, self.logical_size())?
            .end_render_pass()?
            .build()?;

        self.prev_frame = Some(match self.prev_frame.take() {
            Some(last_frame) => self.execute(last_frame.join(acquire), command_buffer)?,
            None => self.execute(acquire, command_buffer)?,
        });
        Ok(())
    }

    fn execute(
        &self,
        acquire_future: impl GpuFuture + Send + Sync + 'static,
        command_buffer: AutoCommandBuffer,
    ) -> Result<Box<GpuFuture + Send + Sync>, failure::Error> {
        let future = acquire_future
            .then_execute(Arc::clone(&self.queue), command_buffer)?
            .then_signal_fence();
        let future = self.target.present(&self.queue, future);
        future.flush()?;
        Ok(future)
    }

    fn logical_size(&self) -> [f32; 2] {
        let [w, h] = self.dimensions;
        [w / self.dpi_factor, h / self.dpi_factor]
    }
}

impl<'a, T> System<'a> for RenderSystem<T>
where
    T: Target,
{
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
