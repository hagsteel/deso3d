use euclid::Rotation3D as Rot3D;
use euclid::{Angle, Transform3D, UnknownUnit};
use gdextras::movement::Move3D;
use gdnative::{Vector2, Vector3};
use legion::prelude::*;
use legion::systems::schedule::Builder;
use serde::{Deserialize, Serialize};

use crate::animation::Animation;
use crate::gameworld::Delta;
use crate::unit::Unit;

type Transform3 = Transform3D<f32, UnknownUnit, UnknownUnit>;
pub type Rotation3 = Rot3D<f32, UnknownUnit, UnknownUnit>;

const GRAVITY: Vector3 = Vector3::new(0., -10., 0.);
const EPSILON: f32 = 1e-4;

fn transform_to_x_y_z_direction(trans: Transform3) -> (Vector3, Vector3, Vector3) {
    let cols = trans.to_column_arrays();
    let v1 = Vector3::new(cols[0][0], cols[0][1], cols[0][2]);
    let v2 = Vector3::new(cols[1][0], cols[1][1], cols[1][2]);
    let v3 = Vector3::new(cols[2][0], cols[2][1], cols[2][2]);

    (v1, v2, v3)
}

pub fn to_2d(v: Vector3) -> Vector2 {
    Vector2::new(v.x, v.z)
}

pub fn to_3d(v: Vector2) -> Vector3 {
    Vector3::new(v.x, 0., v.y)
}

#[derive(Debug, Clone, Copy)]
pub struct Velocity(pub Vector3);

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Pos(pub Vector3);

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct MaxSpeed(pub f32);

#[derive(Debug, Clone, Copy)]
pub struct Destination(pub Vector3);

#[derive(Debug, Clone, Copy)]
pub struct Acceleration(pub Vector3);

pub struct Forces {
    seek: Vector2,
}

impl Forces {
    pub fn zero() -> Self {
        Self {
            seek: Vector2::zero(),
        }
    }
}

// -----------------------------------------------------------------------------
//     - Systems -
// -----------------------------------------------------------------------------

fn reset_acceleration() -> Box<dyn Runnable> {
    SystemBuilder::new("reset acceleration")
        .with_query(<Write<Acceleration>>::query())
        .build_thread_local(|_, world, _, accelerations| {
            for mut acc in accelerations.iter_mut(world) {
                acc.0 = Vector3::zero();
            }
        })
}

fn reset_forces() -> Box<dyn Runnable> {
    SystemBuilder::new("reset_forces")
        .with_query(<Write<Forces>>::query())
        .build_thread_local(|_, world, _, forces| {
            for mut force in forces.iter_mut(world) {
                *force = Forces::zero();
            }
        })
}

fn apply_forces() -> Box<dyn Runnable> {
    SystemBuilder::new("apply_forces")
        .with_query(<(Read<Unit>, Read<Forces>, Write<Acceleration>)>::query())
        .build_thread_local(|_, world, _, query| {
            for (unit, forces, mut acc) in query.iter_mut(world) {
                // acc.0 += forces.separation;

                acc.0 += to_3d(forces.seek);
            }
        })
}

fn seek() -> Box<dyn Runnable> {
    SystemBuilder::new("apply directional velocity")
        .read_resource::<Delta>()
        .with_query(<(
            Read<MaxSpeed>,
            Read<Pos>,
            Read<Destination>,
            Write<Forces>,
            Read<Velocity>,
        )>::query())
        .build_thread_local(|_, world, delta, query| {
            for (max_speed, pos, dest, mut forces, velocity) in query.iter_mut(world) {
                let mut diff = to_2d(dest.0 - pos.0);
                let dist = diff.length();
                let future_dist = to_2d(pos.0 + velocity.0 * delta.0 - dest.0).length();

                if future_dist >= dist {
                    let force = to_2d(-velocity.0) + diff / delta.0;
                    forces.seek = force;
                } else {
                    diff += diff.normalize() * max_speed.0;
                    forces.seek = diff;
                }
            }
        })
}

fn move_units() -> Box<dyn Runnable> {
    SystemBuilder::new("move units")
        .read_resource::<Delta>()
        .with_query(
            <(
                Write<Pos>,
                Write<Unit>,
                Write<Velocity>,
                Read<Acceleration>,
                Read<MaxSpeed>,
            )>::query()
            .filter(component::<Destination>()),
        )
        .build_thread_local(|_, world, delta, units| {
            for (mut pos, mut unit, mut velocity, acc, max_speed) in units.iter_mut(world) {
                velocity.0 += acc.0;
                velocity.0 = velocity.0.with_max_length(max_speed.0);
                velocity.0.y = 0.;
                velocity.0 = unit
                    .inner
                    .move_and_slide_default(velocity.0, Vector3::new(0., 1., 0.));
                pos.0 = unit.translation();
            }
        })
}

fn rotate_unit() -> Box<dyn Runnable> {
    SystemBuilder::new("rotate unit")
        .with_query(<(Write<Unit>, Read<Pos>, Read<Destination>)>::query())
        .build_thread_local(|_, world, _, velocities| {
            for (mut unit, pos, dest) in velocities.iter_mut(world) {
                let diff = dest.0 - pos.0;

                // To stop it flapping we can assume it's done if it's really close
                if diff.length() < 1. {
                    return;
                }

                let direction = diff.normalize();

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

fn done_moving() -> Box<dyn Runnable> {
    SystemBuilder::new("done_moving")
        .with_query(<(Read<Pos>, Read<Destination>, Write<Animation>)>::query())
        .build_thread_local(|cmd, world, _, query| {
            for (ent, (pos, dest, mut animation)) in query.iter_entities_mut(world) {
                let dist = (to_2d(pos.0) - to_2d(dest.0)).length();
                if dist < EPSILON && dist > -EPSILON {
                    cmd.remove_component::<Destination>(ent);
                    *animation = Animation::Idle;
                } else {
                    *animation = Animation::Run;
                }
            }
        })
}

fn apply_gravity() -> Box<dyn Runnable> {
    SystemBuilder::new("apply gravity")
        .with_query(<Write<Unit>>::query())
        .build_thread_local(|cmd, world, resources, units| unsafe {
            for mut unit in units.iter_mut(world) {
                if unit.inner.is_on_floor() {
                    continue;
                }
                unit
                    .inner
                    .move_and_slide_default(GRAVITY, Vector3::new(0., 1., 0.));
            }
    })
}

pub fn movement_systems(builder: Builder) -> Builder {
    builder
        // .add_thread_local(apply_gravity())
        .add_thread_local(reset_acceleration())
        .add_thread_local(reset_forces())
        .add_thread_local(seek())
        .add_thread_local(apply_forces())
        .add_thread_local(rotate_unit())
        .add_thread_local(move_units())
        .add_thread_local(done_moving())
}
