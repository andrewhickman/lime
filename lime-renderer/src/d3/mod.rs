use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::device::Device;
use vulkano::framebuffer::{RenderPassAbstract, Subpass};

use std::sync::Arc;

use Color;

pub struct Mesh;

pub trait Draw {
    fn draw<V: FnMut(&Mesh, Color)>(&self, visitor: V);
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
