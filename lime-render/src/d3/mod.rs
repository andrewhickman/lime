mod geom;
mod mesh;

pub use self::geom::Vector;
pub use self::mesh::Mesh;

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

type Pipeline = Arc<
    GraphicsPipeline<
        SingleBufferDefinition<Vertex>,
        Box<PipelineLayoutAbstract + Send + Sync>,
        Arc<RenderPassAbstract + Send + Sync>,
    >,
>;

pub struct Renderer {
    ubuf: CpuBufferPool<vs::ty::Data>,
    pool: FixedSizeDescriptorSetsPool<Pipeline>,
    pipe: Pipeline,
    queued: Vec<Mesh>,
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
                .vertex_input(SingleBufferDefinition::new())
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                .depth_stencil_simple_depth()
                .render_pass(subpass)
                .build(Arc::clone(device))
                .unwrap_or_else(throw),
        );

        let ubuf = CpuBufferPool::uniform_buffer(Arc::clone(device));
        let pool = FixedSizeDescriptorSetsPool::new(Arc::clone(&pipe), 0);

        Renderer {
            pipe,
            ubuf,
            queued: Vec::new(),
            pool,
        }
    }

    pub(crate) fn commit(
        &mut self,
        mut cmd: AutoCommandBufferBuilder,
        state: &DynamicState,
    ) -> Fallible<AutoCommandBufferBuilder> {
        let ubuf = self.ubuf.next(vs::ty::Data {
            world: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            view: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            proj: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        })?;
        let set = Arc::new(self.pool.next().add_buffer(ubuf)?.build()?);
        for mesh in self.queued.drain(..) {
            cmd = cmd.draw_indexed(
                Arc::clone(&self.pipe),
                state,
                mesh.vertices,
                mesh.indices,
                Arc::clone(&set),
                (),
            )?;
        }
        Ok(cmd)
    }

    pub fn draw_mesh(&mut self, mesh: Mesh) {
        self.queued.push(mesh);
    }
}

#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: Vector,
    normal: Vector,
}

impl Vertex {
    fn new((position, normal): (Vector, Vector)) -> Self {
        Vertex { position, normal }
    }
}

impl_vertex!(Vertex, position, normal);

#[allow(unused)]
mod vs {
    #[derive(VulkanoShader)]
    #[ty = "vertex"]
    #[path = "shader/d3/vert.glsl"]
    struct Dummy;
}

#[allow(unused)]
mod fs {
    #[derive(VulkanoShader)]
    #[ty = "fragment"]
    #[path = "shader/d3/frag.glsl"]
    struct Dummy;
}
