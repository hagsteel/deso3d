use gdextras::input::InputEventExt;
use gdextras::node_ext::NodeExt;
use gdnative::api::{
    AnimationTree as GDAnimationTree, Area, Camera as GodotCamera, CanvasLayer, Control, GridMap,
    InputEvent, InputEventKey, InputEventMouse, InputEventMouseButton, Label, MeshInstance, Node2D,
    Performance, Spatial,
};
use gdnative::{methods, Color, NativeClass, Ptr, Variant, Vector2, Vector3, GodotObject};
use lazy_static::lazy_static;
use legion::prelude::*;
use std::sync::Mutex;

use crate::animation::{animation_systems, Animation, AnimationTree};
use crate::camera::{camera_systems, Camera, Drag, SelectionBox, UnitSelectionArea};
use crate::contextmenu::ContextMenuNode;
use crate::enemy::{enemy_systems, DetectionRange, Enemy};
use crate::formation::{formation_systems, Formation, FormationPos, FormationUI, FormationUnit};
use crate::input::{Keyboard, Keys, MouseButton, MousePos};
use crate::movement::{movement_systems, Acceleration, Forces, MaxSpeed, Pos, Velocity};
use crate::player::{player_systems, PlayerId};
use crate::saveload;
use crate::spawner;
use crate::tilemap::{draw_tilemap, Coords, TileMap};
use crate::unit::Unit;
use crate::safe;

fn setup_physics_schedule() -> Schedule {
    let builder = Schedule::builder();
    let builder = movement_systems(builder);
    let builder = animation_systems(builder);
    builder.build()
}

fn setup_schedule() -> Schedule {
    let builder = Schedule::builder().add_thread_local(draw_tilemap());
    let builder = enemy_systems(builder);
    let builder = camera_systems(builder);
    let builder = player_systems(builder);
    let builder = formation_systems(builder);
    builder.build()
}

// -----------------------------------------------------------------------------
//     - World -
// -----------------------------------------------------------------------------

lazy_static! {
    static ref WORLD: Mutex<World> = Mutex::new(Universe::new().create_world());
}

pub fn with_world<F>(f: F)
where
    F: FnOnce(&mut World),
{
    let _ = WORLD.try_lock().map(|mut world| f(&mut world));
}

// -----------------------------------------------------------------------------
//     - Resources -
// -----------------------------------------------------------------------------
pub struct Delta(pub f32);
pub struct ClickIndicator(pub Ptr<MeshInstance>);

unsafe impl Send for ClickIndicator {}
unsafe impl Sync for ClickIndicator {}

#[derive(Debug)]
pub struct Line(pub Vector3, pub Vector3, pub Color, pub f64);

pub struct DebugLines {
    inner: Vec<Line>,
}

impl DebugLines {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn add(&mut self, start: Vector3, end: Vector3, col: Color, width: f64) {
        self.inner.push(Line(start, end, col, width));
    }
}

#[derive(Debug)]
pub struct ClickedState {
    pub clicked: bool,
}

// -----------------------------------------------------------------------------
//     - Godot node -
// -----------------------------------------------------------------------------

#[derive(NativeClass)]
#[inherit(Spatial)]
pub struct GameWorld {
    resources: Resources,
    physics: Schedule,
    process: Schedule,
}

#[methods]
impl GameWorld {
    pub fn _init(_owner: &Spatial) -> Self {
        let physics = setup_physics_schedule();
        let process = setup_schedule();
        let mut resources = Resources::default();
        resources.insert(Delta(0.));
        resources.insert(MouseButton::Empty);
        resources.insert(MousePos::zero());
        resources.insert(Coords::new());
        resources.insert(Keyboard::new());
        resources.insert(Drag::Empty);
        resources.insert(Formation::new());
        resources.insert(DebugLines::new());
        resources.insert(ClickedState { clicked: false });

        Self {
            resources,
            physics,
            process,
        }
    }

    #[export]
    pub fn _ready(&mut self, owner: &Spatial) {
        let click_indicator = owner.get_and_cast::<MeshInstance>("ClickIndicator");
        self.resources
            .insert(ClickIndicator(click_indicator.claim()));

        // Tilemap
        let gridmap = owner.get_and_cast::<GridMap>("GridMap");
        self.resources.insert(TileMap(gridmap.claim()));

        // Camera
        let camera = owner.get_and_cast::<GodotCamera>("Camera");
        self.resources.insert(Camera(camera.claim()));

        // Unit selection area (detect mouse selection)
        let unit_selection_area = owner.get_and_cast::<Area>("UnitSelectionArea");
        self.resources.insert(UnitSelectionArea(unit_selection_area.claim()));

        // Draw selection node
        let selection_box = owner.get_and_cast::<MeshInstance>("SelectionBox");
        self.resources.insert(SelectionBox(selection_box.claim()));

        // Formation UI
        let formation_ui = spawner::spawn_formation_ui();
        safe!(formation_ui);
        let ui = owner.get_and_cast::<CanvasLayer>("UI");
        ui.add_child(Some(formation_ui.to_node()), false);
        self.resources.insert(FormationUI::new(formation_ui.claim()));

        let colors = [
            Color::rgb(1., 0., 0.),
            Color::rgb(0., 1., 0.),
            Color::rgb(0., 0., 1.),
            Color::rgb(1., 1., 0.),
        ];

        // Player unit
        for i in 0..4 {
            let color_index = i;
            let color = colors[color_index];

            // TODO: remove this
            let formation_x = 0.;
            let formation_y = (i as f32) * 16.;

            let x = (i as f32 + 15.) * 4.;
            let y = 0.4;
            let z = 10.;

            let mut formation_unit = spawner::spawn_formation_unit();
            safe!(formation_unit);
            {
                let p = formation_ui.get_and_cast::<Control>("Pending");
                p.add_child(Some(formation_unit.to_node()), false);
            }
            formation_unit.set_position(Vector2::new(formation_x, formation_y), false);

            let mut unit = spawner::spawn_unit();
            safe!(unit);

            let mut context_menu = spawner::spawn_context_menu();
            safe!(context_menu);

            owner.add_child(Some(unit.to_node()), false);
            unit.set_translation(Vector3::new(x, y, z));
            unit.add_child(Some(context_menu.to_node()), false);

            let pos = unsafe { unit.translation() };

            let anim_tree = unit.get_and_cast::<GDAnimationTree>("AnimationTree");

            let mut unit = Unit::new(unit.claim());
            unit.set_color(color);

            formation_unit.set_modulate(color);
            let mut formation_unit = FormationUnit::new(formation_unit.claim());

            let formation_pos = FormationPos(i as u16);

            let speed = MaxSpeed(7.5f32);

            with_world(|world| {
                world.insert(
                    (PlayerId::new(x as u8),),
                    Some((
                        unit,
                        Velocity(Vector3::zero()),
                        speed,
                        Pos(pos),
                        Forces::zero(),
                        Acceleration(Vector3::zero()),
                        formation_unit,
                        formation_pos,
                        AnimationTree::new(anim_tree.claim()),
                        Animation::Idle,
                        ContextMenuNode(context_menu.claim()),
                    )),
                );
            });
        }

        for x in 15..15 {
            // Remember why you did this? No? Good luck to you!
            let x = x as f32 * 4.;
            let y = 12.;
            let z = 26.;

            let mut unit = spawner::spawn_enemy();
            safe!(unit);
            unsafe {
                owner.add_child(Some(unit.to_node()), false);
                unit.set_translation(Vector3::new(x, y, z));
            }

            let pos = unit.translation();

            let speed = MaxSpeed(10f32);

            with_world(|world| {
                world.insert(
                    (Enemy,),
                    Some((
                        Unit::new(unit.claim()),
                        Velocity(Vector3::zero()),
                        speed,
                        Pos(pos),
                        DetectionRange(10.),
                        Forces::zero(),
                        Acceleration(Vector3::zero()),
                        Animation::Idle,
                    )),
                );
            });
        }
    }

    #[export]
    pub fn _unhandled_input(&mut self, owner: &Spatial, event: Variant) {
        let event = event
            .try_to_object::<InputEvent>()
            .expect("I expect this to be an input event");

        if event.action_pressed("ui_cancel") {
            owner
                .get_tree()
                .map(|tree| unsafe { tree.assume_safe() }.quit(0));
        }

        if event.action_pressed("save") {
            if let Err(e) = saveload::save(0) {
                eprintln!("{:?}", e);
            }
        }

        // Mouse button
        if let Some(btn_event) = event.clone().cast::<InputEventMouseButton>() {
            self.resources.get_mut::<MouseButton>().map(|mut btn| {
                *btn = MouseButton::from_event(btn_event);
            });
        }

        // Mouse pos
        if let Some(mouse_event) = event.clone().cast::<InputEventMouse>() {
            self.resources.get_mut::<MousePos>().map(|mut pos| {
                pos.set_global(mouse_event.global_position());
            });
        }

        // Keyboard
        if let Some(_) = event.clone().cast::<InputEventKey>() {
            self.resources.get_mut::<Keyboard>().map(|mut key| {
                if event.is_action_pressed("Left".into(), false) {
                    key.update(Keys::LEFT, true);
                } else if event.is_action_released("Left".into()) {
                    key.update(Keys::LEFT, false);
                }

                if event.is_action_pressed("Right".into(), false) {
                    key.update(Keys::RIGHT, true);
                } else if event.is_action_released("Right".into()) {
                    key.update(Keys::RIGHT, false);
                }

                if event.is_action_pressed("Up".into(), false) {
                    key.update(Keys::UP, true);
                } else if event.is_action_released("Up".into()) {
                    key.update(Keys::UP, false);
                }

                if event.is_action_pressed("Down".into(), false) {
                    key.update(Keys::DOWN, true);
                } else if event.is_action_released("Down".into()) {
                    key.update(Keys::DOWN, false);
                }
            });
        }
    }

    #[export]
    pub fn _process(&mut self, owner: &Spatial, _: f64) {
        with_world(|world| {
            self.process.execute(world, &mut self.resources);
        });

        // Debug label
        let label = owner.get_and_cast::<Label>("UI/Panel/DebugLabel");
        let perf = Performance::godot_singleton();
        let fps = format!("fps: {}", perf.get_monitor(Performance::TIME_FPS));
        unsafe { label.set_text(fps.into()) };

        self.resources.get_mut::<DebugLines>().map(|mut lines| {
            let dd = owner.get_and_cast::<Node2D>("DebugDraw");
            // dd.with_script(|debug_draw: &mut DebugDraw, _| unsafe {
            //         debug_draw.set_lines(lines.inner.drain(..).collect());
            //         dd.update();
            //     });
            // });
        });
    }

    #[export]
    pub fn _physics_process(&mut self, _: &Spatial, delta: f64) {
        self.resources
            .get_mut::<Delta>()
            .map(|mut d| d.0 = delta as f32);
        with_world(|world| {
            self.physics.execute(world, &mut self.resources);
        });
    }

    // TODO: delete this function (it's in the name)
    pub fn delete_me(&mut self) {
        self.resources
            .get_mut::<ClickedState>()
            .map(|mut s| s.clicked = true);
    }
}
