use gdnative::{KinematicBody, Vector3};

// -----------------------------------------------------------------------------
//     - Components -
// -----------------------------------------------------------------------------
pub struct Unit {
    pub inner: KinematicBody 
}

impl Unit {
    pub fn new(inner: KinematicBody) -> Self {
        Self {
            inner, 
        }
    }

    pub fn translation(&self) -> Vector3 {
        unsafe { self.inner.get_translation() }
    }
}

unsafe impl Send for Unit {}
unsafe impl Sync for Unit {}
