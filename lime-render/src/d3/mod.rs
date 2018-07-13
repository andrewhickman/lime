use std::sync::Arc;

use failure;
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::device::Device;
use vulkano::framebuffer::{RenderPassAbstract, Subpass};

pub struct Mesh;

pub struct Renderer;

impl Renderer {
    pub(crate) fn new(_: &Arc<Device>, _: Subpass<Arc<RenderPassAbstract + Send + Sync>>) -> Self {
        Renderer
    }

    pub(crate) fn draw(
        &self,
        cmd: AutoCommandBufferBuilder,
        _: DynamicState,
    ) -> Result<AutoCommandBufferBuilder, failure::Error> {
        /*self.draw.draw(res, &mut |_, _| ());*/
        Ok(cmd)
    }
}
