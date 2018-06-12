use vulkano::pipeline::vertex::{VertexMember, VertexMemberTy};

#[repr(C)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Point(pub f32, pub f32);

impl Point {
    pub fn origin() -> Self {
        Point(0.0, 0.0)
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        ulps_eq!(self.0, other.0) && ulps_eq!(self.1, other.1)
    }
}

unsafe impl VertexMember for Point {
    fn format() -> (VertexMemberTy, usize) {
        (VertexMemberTy::F32, 2)
    }
}
