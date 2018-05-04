use std::ops::Deref;
use std::sync::Arc;

use specs::shred::Resources;
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::device::Device;
use vulkano::framebuffer::{RenderPassAbstract, Subpass};

use Color;

pub struct Mesh;

pub trait Draw {
    fn draw(&self, res: &Resources, visitor: &mut FnMut(&Mesh, Color));
}

impl<T> Draw for T
where
    T: Deref + ?Sized,
    T::Target: Draw,
{
    fn draw(&self, res: &Resources, visitor: &mut FnMut(&Mesh, Color)) {
        self.deref().draw(res, visitor)
    }
}

pub(crate) struct Renderer;

impl Renderer {
    pub(crate) fn new(_: Arc<Device>, _: Subpass<Arc<RenderPassAbstract + Send + Sync>>) -> Self {
        Renderer
    }

    pub(crate) fn draw<D: Draw>(
        &self,
        res: &Resources,
        cmd: AutoCommandBufferBuilder,
        d3: &D,
        _: DynamicState,
    ) -> AutoCommandBufferBuilder {
        d3.draw(res, &mut |_, _| ());
        cmd
    }
}
