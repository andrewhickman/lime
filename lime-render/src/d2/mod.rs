mod geom;

pub use self::geom::Point;

use std::sync::Arc;

use failure::Fallible;
use utils::throw;
use vulkano::buffer::CpuBufferPool;
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::descriptor::descriptor_set::FixedSizeDescriptorSetsPool;
use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::device::Device;
use vulkano::framebuffer::{RenderPassAbstract, Subpass};
use vulkano::pipeline::vertex::SingleBufferDefinition;
use vulkano::pipeline::GraphicsPipeline;

use Color;

type Pipeline = Arc<
    GraphicsPipeline<
        SingleBufferDefinition<Vertex>,
        Box<PipelineLayoutAbstract + Send + Sync>,
        Arc<RenderPassAbstract + Send + Sync>,
    >,
>;

pub struct Renderer {
    vbuf: CpuBufferPool<Vertex>,
    ubuf: CpuBufferPool<vs::ty::Data>,
    pipe: Pipeline,
    pool: FixedSizeDescriptorSetsPool<Pipeline>,
    queued: Vec<Vertex>,
}

impl Renderer {
    pub(crate) fn new(
        device: &Arc<Device>,
        subpass: Subpass<Arc<RenderPassAbstract + Send + Sync>>,
    ) -> Self {
        let vs = vs::Shader::load(Arc::clone(device)).unwrap_or_else(throw);
        let fs = fs::Shader::load(Arc::clone(device)).unwrap_or_else(throw);

        let pipe = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                .render_pass(subpass)
                .build(Arc::clone(device))
                .unwrap_or_else(throw),
        );

        let vbuf = CpuBufferPool::vertex_buffer(Arc::clone(device));
        let ubuf = CpuBufferPool::uniform_buffer(Arc::clone(device));

        let pool = FixedSizeDescriptorSetsPool::new(Arc::clone(&pipe), 0);

        Renderer {
            pipe,
            vbuf,
            ubuf,
            pool,
            queued: Vec::new(),
        }
    }

    pub(crate) fn commit(
        &mut self,
        cmd: AutoCommandBufferBuilder,
        state: &DynamicState,
        logical_size: [f32; 2],
    ) -> Fallible<AutoCommandBufferBuilder> {
        let vbuf = self.vbuf.chunk(self.queued.drain(..))?;
        let ubuf = self.ubuf.next(vs::ty::Data {
            dimensions: logical_size,
        })?;
        let set = self.pool.next().add_buffer(ubuf)?.build()?;

        Ok(cmd.draw(Arc::clone(&self.pipe), state, vbuf, set, ())?)
    }

    pub fn draw_tri(&mut self, vertices: &[Point], color: Color) {
        debug_assert!(vertices.len() % 3 == 0);
        self.queued
            .extend(vertices.iter().map(|&v| Vertex::new(v, color)));
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
    #[path = "shader/d2/vert.glsl"]
    struct Dummy;
}

#[allow(unused)]
mod fs {
    #[derive(VulkanoShader)]
    #[ty = "fragment"]
    #[path = "shader/d2/frag.glsl"]
    struct Dummy;
}
