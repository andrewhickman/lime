use vulkano::pipeline::vertex::{VertexMember, VertexMemberTy};
use winit::dpi::LogicalPosition;

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

impl From<LogicalPosition> for Point {
    fn from(pos: LogicalPosition) -> Self {
        Point(pos.x as f32, pos.y as f32)
    }
}

unsafe impl VertexMember for Point {
    fn format() -> (VertexMemberTy, usize) {
        (VertexMemberTy::F32, 2)
    }
}
