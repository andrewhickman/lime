#[derive(Copy, Clone, Debug)]
/// Resource holding the current size of the screen.
pub struct ScreenDimensions {
    pub(crate) w: u32,
    pub(crate) h: u32,
}

impl ScreenDimensions {
    pub fn width(&self) -> u32 {
        self.w
    }

    pub fn height(&self) -> u32 {
        self.h
    }
}

impl Into<[u32; 2]> for ScreenDimensions {
    fn into(self) -> [u32; 2] {
        [self.w, self.h]
    }
}

impl From<[u32; 2]> for ScreenDimensions {
    fn from(arr: [u32; 2]) -> Self {
        ScreenDimensions {
            w: arr[0],
            h: arr[1],
        }
    }
}
