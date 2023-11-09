use godot::prelude::*;

mod constraints;
mod rope_2d;
mod rope_engine;
mod rope_point;
struct GodotVerletRope;

#[gdextension]
unsafe impl ExtensionLibrary for GodotVerletRope {}
