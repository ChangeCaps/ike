use ike_ecs::Component;

use crate::NodeStages;

pub trait NodeComponent: Component {
    fn stages(stages: &mut NodeStages);
}
