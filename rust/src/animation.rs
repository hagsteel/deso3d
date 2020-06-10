use gdnative::{AnimationNodeStateMachinePlayback, AnimationTree as GDAnimationTree};
use legion::prelude::*;
use legion::systems::schedule::Builder;

// -----------------------------------------------------------------------------
//     - Components -
// -----------------------------------------------------------------------------

pub enum Animation {
    Idle,
    Run,
}

pub struct AnimationTree(GDAnimationTree);

impl AnimationTree {
    pub fn new(anim_tree: GDAnimationTree) -> Self {
        Self(anim_tree)
    }
}

unsafe impl Send for AnimationTree {}
unsafe impl Sync for AnimationTree {}

// -----------------------------------------------------------------------------
//     - Systems -
// -----------------------------------------------------------------------------

fn animate() -> Box<dyn Runnable> {
    SystemBuilder::new("animate")
        .with_query(<(Read<Animation>, Write<AnimationTree>)>::query())
        .build_thread_local(|cmd, world, resources, animations| {
            for (anim, anim_tree) in animations.iter_mut(world) {
                let playback = unsafe { anim_tree.0.get("parameters/playback".into()) };
                let mut playback = match playback.try_to_object::<AnimationNodeStateMachinePlayback>() {
                    Some(p) => p,
                    None => {
                        eprintln!("{:?}", "failed to get playback");
                        continue;
                    }
                };

                if !playback.is_playing() {
                    playback.start("Idle-loop".into());
                    continue;
                }

                if playback.get_travel_path().len() > 0 {
                    continue;
                }

                // Do actual animation work

                match *anim {
                    Animation::Idle => playback.travel("Idle-loop".into()),
                    Animation::Run => playback.travel("Run-loop".into()),
                }
            }
        },)
}

pub fn animation_systems(builder: Builder) -> Builder {
    builder
       .add_thread_local(animate())
}
