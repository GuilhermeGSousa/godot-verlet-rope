use godot::engine::Node;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct RopeEngine {
    #[base]
    node: Base<Node>,
}

#[godot_api]
impl NodeVirtual for RopeEngine {
    fn init(node: Base<Node>) -> Self {
        Self { node }
    }
}
