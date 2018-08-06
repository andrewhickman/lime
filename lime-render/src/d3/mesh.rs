use std::sync::Arc;

use utils::throw;
use vulkano::buffer::{BufferUsage, ImmutableBuffer};
use vulkano::sync::GpuFuture;

use d3::{Vector, Vertex};
use Context;

#[derive(Clone)]
pub struct Mesh {
    pub(in d3) vertices: Arc<ImmutableBuffer<[Vertex]>>,
    pub(in d3) indices: Arc<ImmutableBuffer<[u16]>>,
}

impl Mesh {
    pub fn new<V, I>(ctx: &Context, vertices: V, indices: I) -> (Self, Box<GpuFuture>)
    where
        V: IntoIterator<Item = (Vector, Vector)>,
        V::IntoIter: ExactSizeIterator,
        I: IntoIterator<Item = u16>,
        I::IntoIter: ExactSizeIterator,
    {
        let (vertices, vertices_future) = ImmutableBuffer::from_iter(
            vertices.into_iter().map(Vertex::new),
            BufferUsage::vertex_buffer(),
            Arc::clone(ctx.transfer_queue()),
        ).unwrap_or_else(throw);
        let (indices, indices_future) = ImmutableBuffer::from_iter(
            indices.into_iter(),
            BufferUsage::index_buffer(),
            Arc::clone(ctx.transfer_queue()),
        ).unwrap_or_else(throw);

        let future = vertices_future.join(indices_future);
        (Mesh { vertices, indices }, Box::new(future))
    }
}
