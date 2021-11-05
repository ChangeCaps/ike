mod components;
mod resources;
mod systems;

pub use components::*;
pub use resources::*;
pub use systems::*;

use ike_core::*;
use rapier3d::prelude::{ColliderSet, JointSet, RigidBodySet};

pub mod physics_stage {
    pub const PRE_PHYSICS: &str = "pre_physics";
    pub const PHYSICS: &str = "physics";
    pub const POST_PHYSICS: &str = "post_physics";
}

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    #[inline]
    fn build(self, app: &mut AppBuilder) {
        app.init_resource::<PhysicsResource>();
        app.insert_resource(RigidBodySet::new());
        app.insert_resource(ColliderSet::new());
        app.insert_resource(JointSet::new());
        app.init_resource::<RigidBodies>();
        app.init_resource::<Colliders>();
        app.init_resource::<Gravity>();

        app.add_stage_after(physics_stage::PRE_PHYSICS, stage::POST_UPDATE);
        app.add_stage_after(physics_stage::PHYSICS, physics_stage::PRE_PHYSICS);
        app.add_stage_after(physics_stage::POST_PHYSICS, physics_stage::PHYSICS);

        app.add_system_to_stage(add_rigid_bodies.system(), physics_stage::PRE_PHYSICS);
        app.add_system_to_stage(add_box_colliders.system(), physics_stage::PRE_PHYSICS);
        app.add_system_to_stage(set_rigid_bodies.system(), physics_stage::PRE_PHYSICS);

        app.add_system_to_stage(physics_update.system(), physics_stage::PHYSICS);

        app.add_system_to_stage(get_rigid_bodies.system(), physics_stage::POST_PHYSICS);
    }
}
