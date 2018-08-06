use vulkano::pipeline::vertex::{VertexMember, VertexMemberTy};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vector(pub f32, pub f32, pub f32);

impl Vector {
    pub fn origin() -> Self {
        Vector(0.0, 0.0, 0.0)
    }
}

impl PartialEq for Vector {
    fn eq(&self, other: &Self) -> bool {
        ulps_eq!(self.0, other.0) && ulps_eq!(self.1, other.1) && ulps_eq!(self.2, other.2)
    }
}

unsafe impl VertexMember for Vector {
    fn format() -> (VertexMemberTy, usize) {
        (VertexMemberTy::F32, 3)
    }
}
