use gdextras::movement::Move3D;
use gdnative::Vector3;
use legion::prelude::*;
use euclid::Rotation3D as Rot3D;
use euclid::{Transform3D, UnknownUnit, Angle};

use crate::unit::Unit;
use crate::gameworld::Delta;
use crate::input::MouseButton;

type Transform3 = Transform3D<f32, UnknownUnit, UnknownUnit>;
pub type Rotation3 = Rot3D<f32, UnknownUnit, UnknownUnit>;

const GRAVITY: f32 = 100.;

// -----------------------------------------------------------------------------
//     - Components -
// -----------------------------------------------------------------------------
#[derive(Debug, Clone, Copy)]
pub struct Velocity(pub Vector3);

#[derive(Debug, Clone, Copy)]
pub struct Pos(pub Vector3);

#[derive(Debug, Clone, Copy)]
pub struct Speed(pub f32);

#[derive(Debug, Clone, Copy)]
pub struct Destination(pub Vector3);

// -----------------------------------------------------------------------------
//     - Systems -
// -----------------------------------------------------------------------------
pub fn apply_gravity() -> Box<dyn Runnable> {
    SystemBuilder::new("apply gravit")
        .read_resource::<Delta>()
        .with_query(<Write<Velocity>>::query())
        .build_thread_local(|cmd, world, delta, velocities| {
            for mut velocity in velocities.iter_mut(world) {
                velocity.0.y -= GRAVITY * delta.0;
            }
        })
}

pub fn apply_directional_velocity() -> Box<dyn Runnable> {
    SystemBuilder::new("apply directional velocity")
        .read_resource::<Delta>()
        .read_resource::<MouseButton>()
        .with_query(<(Write<Velocity>, Read<Speed>, Read<Pos>, Read<Destination>)>::query())
        .build_thread_local(|cmd, world, (delta, mouse_btn), velocities| {
            for (mut velocity, speed, pos, dest) in velocities.iter_mut(world) {
                let mut direction = (dest.0 - pos.0).normalize();
                direction.y = 0.;
                velocity.0 = direction * speed.0;
            }
        })
}

fn transform_to_x_y_z_direction(trans: Transform3) -> (Vector3, Vector3, Vector3) {
    let cols = trans.to_column_arrays();
    let v1 = Vector3::new(cols[0][0], cols[0][1], cols[0][2]);
    let v2 = Vector3::new(cols[1][0], cols[1][1], cols[1][2]);
    let v3 = Vector3::new(cols[2][0], cols[2][1], cols[2][2]);

    (v1, v2, v3)
}

pub fn rotate_unit() -> Box<dyn Runnable> {
    SystemBuilder::new("rotate unit")
        .with_query(<(Write<Unit>, Read<Pos>, Read<Destination>)>::query())
        .build_thread_local(|cmd, world, _, velocities| {
            for (mut unit, pos, dest) in velocities.iter_mut(world) {
                let mut direction = (dest.0 - pos.0).normalize();

                unsafe {
                    let current_rot = unit.inner.get_rotation();
                    let cur_rot = Rotation3::around_y(Angle::radians(current_rot.y));

                    let rot_speed = 0.25;
                    let mut current_transform = unit.inner.get_transform();
                    let angle = Angle::radians(direction.x.atan2(direction.z));
                    let new_rot = Rotation3::around_y(angle);
                    let smooth_rot = cur_rot.slerp(&new_rot, rot_speed);
                    let (x, y, z) = transform_to_x_y_z_direction(smooth_rot.to_transform());

                    current_transform.basis.elements[0] = x;
                    current_transform.basis.elements[1] = y;
                    current_transform.basis.elements[2] = z;

                    unit.inner.set_transform(current_transform);
                }
            }
        })
}

pub fn move_units() -> Box<dyn Runnable> {
    SystemBuilder::new("move units")
        .with_query(<(Write<Pos>, Write<Unit>, Read<Velocity>)>::query())
        .build_thread_local(|cmd, world, res, units| {
            for (entity, (mut pos, mut unit, velocity)) in units.iter_entities_mut(world) {
                unsafe {
                    unit.inner.move_and_slide_default(velocity.0, Vector3::new(0., 1., 0.));
                    pos.0 = unit.translation();
                }
            }
        })
}

pub fn done_moving() -> Box<dyn Runnable> {
    SystemBuilder::new("remove destination if done moving")
        .with_query(<(Read<Destination>, Read<Pos>, Write<Velocity>)>::query())
        .build_thread_local(|cmd, world, res, units| {
            for (entity, (dest, pos, mut velocity)) in units.iter_entities_mut(world) {
                if (dest.0 - pos.0).length() < 2.5 {
                    cmd.remove_component::<Destination>(entity);
                    velocity.0 = Vector3::zero();
                    eprintln!("{:?}", "velocity zero");
                }
            }
    })
}
