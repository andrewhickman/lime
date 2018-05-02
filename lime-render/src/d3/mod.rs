use std::ops::Deref;
use std::sync::Arc;

use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::device::Device;
use vulkano::framebuffer::{RenderPassAbstract, Subpass};

use Color;

pub struct Mesh;

pub trait Draw {
    fn draw(&self, visitor: &mut FnMut(&Mesh, Color));
}

impl<T> Draw for T
where
    T: Deref + ?Sized,
    T::Target: Draw,
{
    fn draw(&self, visitor: &mut FnMut(&Mesh, Color)) {
        self.deref().draw(visitor)
    }
}

pub(crate) struct Renderer;

impl Renderer {
    pub(crate) fn new(_: Arc<Device>, _: Subpass<Arc<RenderPassAbstract + Send + Sync>>) -> Self {
        Renderer
    }

    pub(crate) fn draw<D: Draw>(
        &self,
        cmd: AutoCommandBufferBuilder,
        _: &D,
        _: DynamicState,
    ) -> AutoCommandBufferBuilder {
        cmd
    }
}
