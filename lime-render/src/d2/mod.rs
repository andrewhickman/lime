mod geom;

pub use self::geom::Point;

use std::ops::Deref;
use std::sync::Arc;

use specs::shred::Resources;
use vulkano::buffer::{BufferUsage, CpuBufferPool};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::device::Device;
use vulkano::framebuffer::{RenderPassAbstract, Subpass};
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::vertex::SingleBufferDefinition;

use {quit, Color};

pub trait Draw {
    fn draw(&self, res: &Resources, visitor: &mut FnMut(&[Point], Color));
}

impl<T: ?Sized> Draw for T
where
    T: Deref,
    T::Target: Draw,
{
    fn draw(&self, res: &Resources, visitor: &mut FnMut(&[Point], Color)) {
        self.deref().draw(res, visitor)
    }
}

pub(crate) struct Renderer {
    cpu_buf: CpuBufferPool<Vertex>,
    pipe: Arc<
        GraphicsPipeline<
            SingleBufferDefinition<Vertex>,
            Box<PipelineLayoutAbstract + Send + Sync>,
            Arc<RenderPassAbstract + Send + Sync>,
        >,
    >,
}

impl Renderer {
    pub(crate) fn new(
        device: Arc<Device>,
        subpass: Subpass<Arc<RenderPassAbstract + Send + Sync>>,
    ) -> Self {
        let vs = vs::Shader::load(Arc::clone(&device)).unwrap_or_else(quit);
        let fs = fs::Shader::load(Arc::clone(&device)).unwrap_or_else(quit);

        let pipe = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                .render_pass(subpass)
                .build(Arc::clone(&device))
                .unwrap_or_else(quit),
        );

        let usage = BufferUsage::vertex_buffer_transfer_destination();
        let cpu_buf = CpuBufferPool::new(device, usage);

        Renderer { pipe, cpu_buf }
    }

    pub(crate) fn draw<D: Draw>(
        &self,
        res: &Resources,
        cmd: AutoCommandBufferBuilder,
        draw: &D,
        state: DynamicState,
    ) -> AutoCommandBufferBuilder {
        let mut vx_buf = Vec::new();
        draw.draw(res, &mut |vertices, color| {
            debug_assert!(vertices.len() % 3 == 0);
            vx_buf.extend(vertices.iter().map(|&vx| Vertex::new(vx, color)));
        });
        let buf = self.cpu_buf.chunk(vx_buf).unwrap_or_else(quit);
        cmd.draw(Arc::clone(&self.pipe), state, buf, (), ())
            .unwrap_or_else(quit)
    }
}

#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: Point,
    color: Color,
}

impl Vertex {
    fn new(position: Point, color: Color) -> Self {
        Vertex { position, color }
    }
}

impl_vertex!(Vertex, position, color);

#[allow(unused)]
mod vs {
    #[derive(VulkanoShader)]
    #[ty = "vertex"]
    #[path = "shader/vert2d.glsl"]
    struct Dummy;
}

#[allow(unused)]
mod fs {
    #[derive(VulkanoShader)]
    #[ty = "fragment"]
    #[path = "shader/frag2d.glsl"]
    struct Dummy;
}
