use std::ops::Range;
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

use d2::Point;
use Color;

type Pipeline = Arc<
    GraphicsPipeline<
        SingleBufferDefinition<Vertex>,
        Box<PipelineLayoutAbstract + Send + Sync>,
        Arc<RenderPassAbstract + Send + Sync>,
    >,
>;

pub(in d2) struct TriangleBrush {
    vbuf: CpuBufferPool<Vertex>,
    ubuf: CpuBufferPool<vs::ty::Data>,
    pipe: Pipeline,
    pool: FixedSizeDescriptorSetsPool<Pipeline>,
    queued: Vec<Vertex>,
}

#[derive(Clone, Debug)]
pub(in d2) struct TriangleSection {
    range: Range<usize>,
}

impl TriangleBrush {
    pub(in d2) fn new(
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

        TriangleBrush {
            pipe,
            vbuf,
            ubuf,
            pool,
            queued: Vec::new(),
        }
    }

    pub(in d2) fn draw(
        &mut self,
        cmd: AutoCommandBufferBuilder,
        section: &TriangleSection,
        state: &DynamicState,
        logical_size: [f32; 2],
    ) -> Fallible<AutoCommandBufferBuilder> {
        if section.range.len() == 0 {
            return Ok(cmd);
        }

        let vbuf = self
            .vbuf
            .chunk(self.queued[section.range.clone()].iter().cloned())?;
        let ubuf = self.ubuf.next(vs::ty::Data {
            dimensions: logical_size,
        })?;
        let set = self.pool.next().add_buffer(ubuf)?.build()?;

        Ok(cmd.draw(Arc::clone(&self.pipe), state, vbuf, set, ())?)
    }

    pub(in d2) fn queue_tris(&mut self, vertices: &[Point], color: Color) -> TriangleSection {
        debug_assert!(vertices.len() % 3 == 0);
        let start = self.queued.len();
        self.queued
            .extend(vertices.iter().map(|&v| Vertex::new(v, color)));
        let end = self.queued.len();
        TriangleSection { range: start..end }
    }
}

impl TriangleSection {
    pub(in d2) fn append(&mut self, next: &TriangleSection) {
        debug_assert_eq!(self.range.end, next.range.start);
        self.range.end = next.range.end;
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
