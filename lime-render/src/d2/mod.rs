mod geom;

pub use self::geom::Point;

use std::ops::Deref;
use std::sync::Arc;

use failure;
use specs::shred::Resources;
use utils::throw;
use vulkano::buffer::{BufferUsage, CpuBufferPool};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::descriptor::descriptor_set::FixedSizeDescriptorSetsPool;
use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::device::Device;
use vulkano::framebuffer::{RenderPassAbstract, Subpass};
use vulkano::pipeline::vertex::SingleBufferDefinition;
use vulkano::pipeline::GraphicsPipeline;

use Color;

pub trait Draw: 'static {
    fn draw(&self, res: &Resources, visitor: &mut FnMut(&[Point], Color));
}

impl<T: ?Sized + 'static> Draw for T
where
    T: Deref,
    T::Target: Draw,
{
    fn draw(&self, res: &Resources, visitor: &mut FnMut(&[Point], Color)) {
        self.deref().draw(res, visitor)
    }
}

type Pipeline = Arc<
    GraphicsPipeline<
        SingleBufferDefinition<Vertex>,
        Box<PipelineLayoutAbstract + Send + Sync>,
        Arc<RenderPassAbstract + Send + Sync>,
    >,
>;

pub(crate) struct Renderer {
    vbuf: CpuBufferPool<Vertex>,
    ubuf: CpuBufferPool<vs::ty::Data>,
    pipe: Pipeline,
    pool: FixedSizeDescriptorSetsPool<Pipeline>,
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

        let vbuf = {
            let usage = BufferUsage::vertex_buffer();
            CpuBufferPool::new(Arc::clone(device), usage)
        };

        let ubuf = {
            let usage = BufferUsage::uniform_buffer();
            CpuBufferPool::new(Arc::clone(device), usage)
        };

        let pool = FixedSizeDescriptorSetsPool::new(Arc::clone(&pipe), 0);

        Renderer {
            pipe,
            vbuf,
            ubuf,
            pool,
        }
    }

    pub(crate) fn draw<D: Draw>(
        &mut self,
        res: &Resources,
        cmd: AutoCommandBufferBuilder,
        draw: &D,
        state: DynamicState,
    ) -> Result<AutoCommandBufferBuilder, failure::Error> {
        let mut vbuf = Vec::new();
        draw.draw(res, &mut |vertices, color| {
            debug_assert!(vertices.len() % 3 == 0);
            vbuf.extend(vertices.iter().map(|&v| Vertex::new(v, color)));
        });
        let vbuf = self.vbuf.chunk(vbuf)?;
        let ubuf = self.ubuf.next(vs::ty::Data {
            dimensions: state.viewports.as_ref().unwrap()[0].dimensions,
        })?;
        let set = self.pool.next().add_buffer(ubuf)?.build()?;

        Ok(cmd.draw(Arc::clone(&self.pipe), state, vbuf, set, ())?)
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
