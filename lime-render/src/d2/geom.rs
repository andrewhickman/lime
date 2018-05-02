use vulkano::pipeline::vertex::{VertexMember, VertexMemberTy};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Point(pub f32, pub f32);

unsafe impl VertexMember for Point {
    fn format() -> (VertexMemberTy, usize) {
        (VertexMemberTy::F32, 2)
    }
}
