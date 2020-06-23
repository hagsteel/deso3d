use gdextras::node_ext::NodeExt;

use gdnative::api::{Control, InputEvent, InputEventMouseButton, Spatial};
use gdnative::{
    godot_error, godot_wrap_method, godot_wrap_method_inner, godot_wrap_method_parameter_count,
    methods, Variant, NativeClass, Ptr
};

use crate::gameworld::GameWorld;

// -----------------------------------------------------------------------------
//     - Component -
// -----------------------------------------------------------------------------
pub struct ContextMenuNode(pub Ptr<Control>);

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
    pub fn _init(_owner: &Control) -> Self {
        Self {}
    }

    #[export]
    pub fn selected_option(&self, owner: &Control, event: Variant, value: i64) {
        let event = event.try_to_object::<InputEvent>().expect("Was not an input event. (how?)");
        if let Some(ev) = event.cast::<InputEventMouseButton>() {
            if ev.is_pressed() {
                self.select_option(owner, value);
            }
        }
    }

    fn select_option(&self, owner: &Control, _value: i64) -> Option<()> {
        let tree = unsafe { owner.get_tree()?.assume_safe_during(self) };
        let gameworld = unsafe { tree.root()?.assume_safe_during(self) }
            .get_and_cast::<Spatial>("GameWorld");

        // TODO: this is a terrible hack. Rethink this... please
        // but don't give up on this, just come up with an interface that makes
        // sense and doesn't ruin everything
        // gameworld.with_script(|game: &mut GameWorld, _| {
        //     game.delete_me();
        // });

        Some(())
    }
}
