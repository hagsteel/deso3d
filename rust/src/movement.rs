use euclid::Rotation3D as Rot3D;
use euclid::{Angle, Transform3D, UnknownUnit};
use gdextras::movement::Move3D;
use gdnative::{Vector2, Vector3};
use legion::prelude::*;
use legion::systems::schedule::Builder;
use serde::{Deserialize, Serialize};

use crate::gameworld::{Delta, A, B};
use crate::unit::Unit;

type Transform3 = Transform3D<f32, UnknownUnit, UnknownUnit>;
pub type Rotation3 = Rot3D<f32, UnknownUnit, UnknownUnit>;

const GRAVITY: Vector3 = Vector3::new(0., -10., 0.);
const EPSILON: f32 = 1e-1;

fn transform_to_x_y_z_direction(trans: Transform3) -> (Vector3, Vector3, Vector3) {
    let cols = trans.to_column_arrays();
    let v1 = Vector3::new(cols[0][0], cols[0][1], cols[0][2]);
    let v2 = Vector3::new(cols[1][0], cols[1][1], cols[1][2]);
    let v3 = Vector3::new(cols[2][0], cols[2][1], cols[2][2]);

    (v1, v2, v3)
}

fn to_2d(v: Vector3) -> Vector2 {
    Vector2::new(v.x, v.z)
}

fn to_3d(v: Vector2) -> Vector3 {
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
    separation: Vector3,
    seek: Vector3,
}

impl Forces {
    pub fn zero() -> Self {
        Self {
            separation: Vector3::zero(),
            seek: Vector3::zero(),
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
        .read_resource::<Delta>()
        .with_query(<(Read<Unit>, Read<Forces>, Write<Acceleration>)>::query())
        .build_thread_local(|_, world, delta, query| {
            for (unit, forces, mut acc) in query.iter_mut(world) {
                // Add gravity
                unsafe {
                    if unit.inner.is_on_floor() {
                        acc.0.y = GRAVITY.y * delta.0;
                    } else {
                        acc.0.y += GRAVITY.y * delta.0;
                    }
                }

                // acc.0 += forces.separation;
                acc.0 += forces.seek;
            }
        })
}

fn map_val(x: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
}

fn map_vec(v: Vector2, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> Vector2 {
    Vector2::new(
        map_val(v.x, in_min, in_max, out_min, out_max),
        map_val(v.y, in_min, in_max, out_min, out_max),
    )
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
        .build_thread_local(|cmd, world, delta, query| {
            for (ent, (max_speed, pos, dest, mut forces, velocity)) in
                query.iter_entities_mut(world)
            {
                let mut diff = to_2d(dest.0 - pos.0);
                let dist = diff.length();

                diff += diff.normalize() * max_speed.0;
                forces.seek = to_3d(diff);

                // let attraction = a.0 / dist.powf(2.);
                // let repulsive = b.0 / dist.powf(5.);
                // eprintln!("{:?}", velocity.0.normalize(), direction);
                // forces.seek = direction * attractive + velocity.0 * repulsive;

                //direction * (attractive + repulsive);

                // various forces -> acceleration -> added to the velocity

                // velocity.0 / (dist * max_speed)

                // if dist <= 2. {
                //     eprintln!("huzzah: {}", dist);
                //     forces.seek = to_3d(direction) * attraction + to_3d(direction) * velocity.0.length() * repulsive;
                // } else if dist <= 5. {
                //     // forces.seek = direction * attraction + direction * velocity.0.length() * repulsive;
                //     eprintln!("{:?}", dist);
                // } else {
                //     forces.seek = to_3d(diff).with_max_length(max_speed.0);
                // }

                // forces.seek.y = 0.; // redundant
            }
        })
}

fn move_units() -> Box<dyn Runnable> {
    SystemBuilder::new("move units")
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
        .build_thread_local(|_, world, _, units| {
            for (mut pos, mut unit, mut velocity, acc, max_speed) in units.iter_mut(world) {
                velocity.0 += acc.0;
                velocity.0 = velocity.0.with_max_length(max_speed.0);
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
                let direction = (dest.0 - pos.0).normalize();

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
        .with_query(<(Read<Pos>, Read<Destination>, Read<Velocity>)>::query())
        .build_thread_local(|cmd, world, resources, query| {
            for (ent, (pos, dest, vel)) in query.iter_entities(world) {
                let dist = to_2d(pos.0 - dest.0).length();
                if dist < EPSILON && dist > -EPSILON {
                    eprintln!("{:?}", "done");
                    cmd.remove_component::<Destination>(ent);
                } else {
                    eprintln!("{}", dist);
                }
            }
        })
}

pub fn movement_systems(builder: Builder) -> Builder {
    builder
        .add_thread_local(reset_acceleration())
        .add_thread_local(reset_forces())
        .add_thread_local(seek())
        .add_thread_local(apply_forces())
        // .add_thread_local(apply_directional_velocity())
        // .add_thread_local(apply_separation())
        .add_thread_local(rotate_unit())
        .add_thread_local(move_units())
        .add_thread_local(done_moving())
}
