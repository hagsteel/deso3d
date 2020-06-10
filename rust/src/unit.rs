use gdextras::some_or_bail;
use gdextras::node_ext::NodeExt;
use gdnative::{KinematicBody, Vector3, Color, MeshInstance, SpatialMaterial};

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

    pub fn set_color(&mut self, color: Color) {
        unsafe {
            let mut mesh = some_or_bail!(
                self.inner.get_and_cast::<MeshInstance>("Armature/Skeleton/Humanoid"),
                "failed to get unit mesh"
            );

            let mut spatial_mat = SpatialMaterial::new();
            spatial_mat.set_albedo(color);
            mesh.set_material_override(Some(spatial_mat.to_material()));
        }
    }
}

unsafe impl Send for Unit {}
unsafe impl Sync for Unit {}
