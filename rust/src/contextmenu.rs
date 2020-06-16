use gdextras::node_ext::NodeExt;
use gdnative::{
    godot_error, godot_wrap_method, godot_wrap_method_inner, godot_wrap_method_parameter_count,
    methods, Control, InputEvent, InputEventMouseButton, NativeClass, Spatial
};

use crate::gameworld::GameWorld;

// -----------------------------------------------------------------------------
//     - Component -
// -----------------------------------------------------------------------------
pub struct ContextMenuNode(pub Control);

unsafe impl Send for ContextMenuNode {}
unsafe impl Sync for ContextMenuNode {}

// -----------------------------------------------------------------------------
//     - Godot script -
// -----------------------------------------------------------------------------

#[derive(NativeClass)]
#[inherit(Control)]
pub struct ContextMenu {}

#[methods]
impl ContextMenu {
    pub fn _init(_owner: Control) -> Self {
        Self {}
    }

    #[export]
    pub fn selected_option(&self, owner: Control, event: InputEvent, value: i64) {
        if let Some(ev) = event.cast::<InputEventMouseButton>() {
            if ev.is_pressed() {
                unsafe { self.select_option(owner, value) };
            }
        }
    }

    unsafe fn select_option(&self, owner: Control, _value: i64) -> Option<()> {
        let mut gameworld = owner.get_tree()?.get_root()?.get_and_cast::<Spatial>("GameWorld")?;
        // TODO: this is a terrible hack. Rethink this... please
        // but don't give up on this, just come up with an interface that makes 
        // sense and doesn't ruin everything

        gameworld.with_script(|game: &mut GameWorld, _| {
            game.delete_me();
        });

        Some(())
    }
}
