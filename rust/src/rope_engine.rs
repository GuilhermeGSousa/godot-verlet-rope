use godot::engine::Node;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct RopeEngine {
    #[base]
    node: Base<Node>,

    #[export]
    pub iteration_count: i32,
}

#[godot_api]
impl RopeEngine {}

#[godot_api]
impl NodeVirtual for RopeEngine {
    fn init(node: Base<Node>) -> Self {
        Self {
            node,
            iteration_count: 50,
        }
    }
}
