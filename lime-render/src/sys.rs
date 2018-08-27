use std::marker::PhantomData;
use std::sync::Arc;

use failure::Fallible;
use shrev::{EventChannel, ReaderId};
use specs::prelude::*;
use utils::throw;
use vulkano::command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder, DynamicState};
use vulkano::framebuffer::Subpass;
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::pipeline::viewport::Viewport;
use vulkano::sync::GpuFuture;
use vulkano_win;
use winit::{self, WindowEvent};

use {d2, d3, Context, Target};

pub(crate) struct RenderSystem<T> {
    prev_frame: Option<Box<GpuFuture + Send + Sync>>,
    swapchain_dirty: bool,
    event_rx: ReaderId<winit::Event>,
    state: DynamicState,
    _target: PhantomData<T>,
}

impl<T> RenderSystem<T>
where
    T: Target,
{
    pub const NAME: &'static str = "render::Render";

    pub(crate) fn add(
        world: &mut World,
        dispatcher: &mut DispatcherBuilder,
        data: T::InitData,
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

        let (target, ctx) = T::new(phys, data).unwrap_or_else(throw);

        let event_rx = world
            .write_resource::<EventChannel<winit::Event>>()
            .register_reader();

        let [w, h] = target.dimensions();
        let state = DynamicState {
            line_width: None,
            viewports: Some(vec![Viewport {
                origin: [0.0, 0.0],
                dimensions: [w as f32, h as f32],
                depth_range: 0.0..1.0,
            }]),
            scissors: None,
        };

        world.add_resource(d3::Renderer::new(
            ctx.device(),
            Subpass::from(Arc::clone(target.render_pass()), 0).unwrap(),
        ));
        world.add_resource(d2::Renderer::new(
            ctx.device(),
            Subpass::from(Arc::clone(target.render_pass()), 1).unwrap(),
        ));
        world.add_resource(ctx);
        world.add_resource(target);

        dispatcher.add(
            RenderSystem {
                prev_frame: None,
                swapchain_dirty: false,
                event_rx,
                state,
                _target: PhantomData::<T>,
            },
            Self::NAME,
            &[d3_sys, d2_sys],
        )
    }

    fn render(
        &mut self,
        ctx: &Context,
        target: &mut T,
        d3: &mut d3::Renderer,
        d2: &mut d2::Renderer,
    ) {
        for _ in 0..5 {
            if self.swapchain_dirty {
                match self.recreate_swapchain(ctx, target) {
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
                match self.try_render(ctx, target, d3, d2) {
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

    fn recreate_swapchain(&mut self, ctx: &Context, target: &mut T) -> Fallible<()> {
        target.recreate(ctx)?;
        let [w, h] = target.dimensions();
        self.state.viewports.as_mut().unwrap()[0].dimensions = [w as f32, h as f32];
        Ok(())
    }

    fn try_render(
        &mut self,
        ctx: &Context,
        target: &mut T,
        d3: &mut d3::Renderer,
        d2: &mut d2::Renderer,
    ) -> Fallible<()> {
        let (fb, acquire) = target.acquire(ctx)?;

        if let Some(ref mut last_frame) = self.prev_frame {
            last_frame.cleanup_finished();
        }

        let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(
            Arc::clone(ctx.device()),
            ctx.graphics_queue().family(),
        )?.begin_render_pass(
            fb,
            false,
            vec![[0.0, 0.0, 0.0, 1.0].into(), 1f32.into()],
        )?;
        let command_buffer = d3.commit(command_buffer, &self.state)?.next_subpass(false)?;
        let command_buffer = d2
            .commit(command_buffer, &self.state, target.logical_size())?
            .end_render_pass()?
            .build()?;

        self.prev_frame = Some(match self.prev_frame.take() {
            Some(last_frame) => {
                self.execute(ctx, target, last_frame.join(acquire), command_buffer)?
            }
            None => self.execute(ctx, target, acquire, command_buffer)?,
        });
        Ok(())
    }

    fn execute(
        &mut self,
        ctx: &Context,
        target: &mut T,
        acquire_future: impl GpuFuture + Send + Sync + 'static,
        command_buffer: AutoCommandBuffer,
    ) -> Fallible<Box<GpuFuture + Send + Sync>> {
        let future = acquire_future
            .then_execute(Arc::clone(ctx.graphics_queue()), command_buffer)?
            .then_signal_fence();
        let future = target.present(ctx, future)?;
        future.flush()?;
        Ok(future)
    }
}

impl<'a, T> System<'a> for RenderSystem<T>
where
    T: Target,
{
    type SystemData = (
        ReadExpect<'a, EventChannel<winit::Event>>,
        ReadExpect<'a, Context>,
        WriteExpect<'a, T>,
        WriteExpect<'a, d3::Renderer>,
        WriteExpect<'a, d2::Renderer>,
    );

    fn run(&mut self, (event_tx, ctx, mut target, mut d3, mut d2): Self::SystemData) {
        for event in event_tx.read(&mut self.event_rx) {
            if let winit::Event::WindowEvent {
                event: WindowEvent::HiDpiFactorChanged(_factor),
                ..
            } = event
            {
                //                self.target.set_hidpi_factor(factor)
            }
        }

        self.render(&ctx, &mut target, &mut d3, &mut d2);
    }
}
