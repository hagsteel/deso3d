use gdextras::node_ext::NodeExt;
use gdnative::api::{KinematicBody, MeshInstance, SpatialMaterial};
use gdnative::{Color, Ptr, Vector3};

// -----------------------------------------------------------------------------
//     - Components -
// -----------------------------------------------------------------------------
pub struct Unit {
    pub inner: Ptr<KinematicBody>,
}

impl Unit {
    pub fn new(inner: Ptr<KinematicBody>) -> Self {
        Self { inner }
    }

    pub fn translation(&self) -> Vector3 {
        unsafe { self.inner.assume_safe_during(self).translation() }
    }

    pub fn set_color(&mut self, color: Color) {
        let mesh = unsafe { self.inner.assume_safe() }
            .get_and_cast::<MeshInstance>("Armature/Skeleton/Humanoid");
        let spatial_mat = SpatialMaterial::new();
        spatial_mat.set_albedo(color);
        mesh.set_material_override(Some(spatial_mat.to_material()));
    }
}

unsafe impl Send for Unit {}
unsafe impl Sync for Unit {}
