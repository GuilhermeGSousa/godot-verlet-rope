use crate::constraints::{Constraint, DistanceContraint, Node2DPinContraint, PinConstraint};
use crate::rope_engine::RopeEngine;
use crate::rope_point::RopeParticle;
use godot::engine::{
    Area2D, Area2DVirtual, CircleShape2D, Line2D, PhysicsBody2D, PhysicsServer2D, ProjectSettings,
    RigidBody2D, Shape2D, StaticBody2D,
};
use godot::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(GodotClass)]
#[class(base=Area2D)]
struct Rope2D {
    #[base]
    base: Base<Area2D>,

    #[export]
    line_renderer: Option<Gd<Line2D>>,

    #[export]
    pin_indices: Array<i32>,

    #[export]
    segment_distance: f32,

    #[export]
    use_colliders: bool,
    collision_particles: Vec<Gd<CircleShape2D>>,
    rope_particles: Vec<Rc<RefCell<RopeParticle>>>,
    constraints: Vec<Box<dyn Constraint>>,
    colliding_bodies: Vec<Gd<PhysicsBody2D>>,
    iteration_count: i32,
    // engine specific
    gravity_vector: Vector2,
    gravity: f32,
}

#[godot_api]
impl Rope2D {
    #[func]
    pub fn bind_to_rope(&mut self, mut other_rope: Gd<Rope2D>, self_index: u32, other_index: u32) {
        match (
            self.rope_particles.get(self_index as usize),
            other_rope
                .bind_mut()
                .rope_particles
                .get(other_index as usize),
        ) {
            (Some(rope_point), Some(other_point)) => self.add_constraint(DistanceContraint::new(
                Rc::downgrade(rope_point),
                Rc::downgrade(other_point),
                1.0,
            )),
            _ => (),
        };
    }

    #[func]
    pub fn bind_to_node(&mut self, node: Gd<Node2D>, at_index: u32) {
        if let Some(rope_point) = self.rope_particles.get(at_index as usize) {
            self.add_constraint(Node2DPinContraint::new(Rc::downgrade(rope_point), node));
        }
    }

    fn add_constraint(&mut self, constraint: impl Constraint + 'static) {
        self.constraints.push(Box::new(constraint));
    }

    fn solve_constraints(&mut self) {
        for _ in 0..self.iteration_count {
            for constraint in &mut self.constraints {
                // Solve
                constraint.solve();
            }
        }
    }

    fn get_rope_engine(&self) -> Gd<RopeEngine> {
        self.base
            .get_node_as::<RopeEngine>("/root/GlobalRopeEngine")
    }

    fn initialize(&mut self) {
        let engine = self.get_rope_engine();
        self.iteration_count = engine.bind().iteration_count;
        self.rope_particles.clear();

        let mut physics_server = PhysicsServer2D::singleton();
        let line_thickness = self.line_renderer.as_ref().unwrap().get_width();
        for point in self.line_renderer.as_ref().unwrap().get_points().as_slice() {
            self.rope_particles.push(Rc::new(RefCell::new(RopeParticle {
                position: point.clone(),
                prev_position: point.clone(),
            })));

            if !self.use_colliders {
                continue;
            }
            let mut particle_shape = CircleShape2D::new();
            particle_shape.set_radius(line_thickness / 2.0);
            let shape_rid = particle_shape.get_rid();
            physics_server.area_add_shape(self.base.get_rid(), shape_rid);
            self.collision_particles.push(particle_shape);
        }

        // Add Distance Constraints
        for i in 0..self.rope_particles.len() - 1 {
            self.add_constraint(DistanceContraint::new(
                Rc::downgrade(&self.rope_particles[i]),
                Rc::downgrade(&self.rope_particles[i + 1]),
                self.segment_distance,
            ));
        }

        // Add Pin Contraints
        for i in 0..self.pin_indices.len().clone() {
            let pin_index = self.pin_indices.get(i) as usize;
            let rope_point_opt = self.rope_particles.get(pin_index);
            if let Some(rope_point) = rope_point_opt {
                let rope_point = rope_point.clone();
                let position = rope_point.borrow().position;
                self.add_constraint(PinConstraint::new(position, rope_point));
            }
        }
    }

    fn update_position(&mut self, delta_time: f32) {
        let gravity_displacement =
            0.5 * delta_time * delta_time * self.gravity * self.gravity_vector;

        for p in &mut self.rope_particles {
            let velocity = p.borrow().get_velocity(delta_time);
            let current_position =
                p.borrow().position + velocity * delta_time + gravity_displacement;
            p.borrow_mut().update_position(current_position);
        }
    }

    fn render(&mut self) {
        let line_renderer: &mut Gd<Line2D> = self.line_renderer.as_mut().unwrap();

        line_renderer.clear_points();

        for p in &self.rope_particles {
            line_renderer.add_point(p.borrow().position);
        }
    }

    fn update_collision_shape(&mut self) {
        if !self.use_colliders {
            return;
        }
        let mut physics_server = PhysicsServer2D::singleton();
        for i in 0..self.rope_particles.len() {
            let rope_particle = &self.rope_particles[i];

            let xform = self
                .base
                .get_transform()
                .translated_local(rope_particle.borrow().position);
            physics_server.area_set_shape_transform(self.base.get_rid(), i as i32, xform);
        }
    }

    fn solve_collisions(&mut self, delta_time: f32) {
        if !self.use_colliders {
            return;
        }

        let mut colliding_shapes: Vec<(Gd<PhysicsBody2D>, Gd<Shape2D>)> = Vec::new();
        for body in &mut self.colliding_bodies {
            let shape_owners = body.get_shape_owners();
            for owner_id in shape_owners.as_slice() {
                let shape_count = body.shape_owner_get_shape_count(*owner_id as u32);
                for shape_id in 0..shape_count {
                    if let Some(shape) = body.shape_owner_get_shape(*owner_id as u32, shape_id) {
                        colliding_shapes.push((body.clone(), shape));
                    };
                }
            }
        }
        for shape in &mut colliding_shapes {
            self.apply_collisions_with_shape(&mut shape.0, &mut shape.1, delta_time);
        }
    }

    fn apply_collisions_with_shape(
        &mut self,
        other_body: &mut Gd<PhysicsBody2D>,
        other_shape: &mut Gd<Shape2D>,
        delta_time: f32,
    ) {
        for shape_idx in 0..self.collision_particles.len() {
            let rope_particle = &self.rope_particles[shape_idx];
            let particle_shape = &mut self.collision_particles[shape_idx];
            let xform = self
                .base
                .get_transform()
                .translated_local(rope_particle.borrow().position);

            let contacts = other_shape.collide_and_get_contacts(
                other_body.get_transform(),
                particle_shape.clone().upcast(),
                xform,
            );
            for contact_idx in (0..contacts.len()).step_by(2) {
                let contact_a = contacts.get(contact_idx);
                let contact_b = contacts.get(contact_idx + 1);
                let contact_vector = contact_b - contact_a;

                // Solving for rigid bodies
                let rigid_body_2d = other_body.clone().try_cast::<RigidBody2D>();
                if let Some(mut rigid_body_2d) = rigid_body_2d {
                    // This is a simplification, the normalized mass should be computed based on the mass of the rope
                    // particle and that of the rigid body. Since our rope particles have no concept of mass
                    // we'll just say they weight the same
                    let normalized_mass = 0.5;
                    let new_pos =
                        rope_particle.borrow().position - contact_vector * normalized_mass;
                    rope_particle.borrow_mut().position = new_pos;
                    let new_rb_vel = contact_vector * (0.1) / delta_time;

                    //rigid_body_2d.set_linear_velocity(new_rb_vel);

                    let mass = rigid_body_2d.get_mass();
                    rigid_body_2d.apply_impulse(new_rb_vel * mass);
                }

                // Solving for static bodies
                let static_body_opt = other_body.clone().try_cast::<StaticBody2D>();
                if let Some(_) = static_body_opt {
                    let new_pos = rope_particle.borrow().position - contact_vector;
                    rope_particle.borrow_mut().position = new_pos;
                }
            }
        }
    }

    #[func]
    fn body_entered(&mut self, node: Gd<PhysicsBody2D>) {
        self.colliding_bodies.push(node);
    }

    #[func]
    fn body_exited(&mut self, node: Gd<PhysicsBody2D>) {
        self.colliding_bodies.retain(|x| *x != node);
    }
}

#[godot_api]
impl Area2DVirtual for Rope2D {
    fn init(base: Base<Area2D>) -> Self {
        let project_settings = ProjectSettings::singleton();
        Self {
            base: base,
            line_renderer: None,
            pin_indices: Array::new(),
            use_colliders: false,
            rope_particles: Vec::new(),
            segment_distance: 100.0,
            collision_particles: Vec::new(),
            constraints: Vec::new(),
            colliding_bodies: Vec::new(),
            iteration_count: 50,
            gravity_vector: project_settings
                .get_setting("physics/2d/default_gravity_vector".into())
                .to(),
            gravity: project_settings
                .get_setting("physics/2d/default_gravity".into())
                .to(),
        }
    }

    fn enter_tree(&mut self) {
        let body_entered = self.base.callable("body_entered");
        let body_exited = self.base.callable("body_exited");

        self.base.connect("body_entered".into(), body_entered);
        self.base.connect("body_exited".into(), body_exited);

        self.initialize();
        self.update_collision_shape();
    }

    fn ready(&mut self) {}

    fn physics_process(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;
        self.update_position(delta_time);
        self.solve_constraints();
        self.solve_collisions(delta_time);
        self.update_collision_shape();
        self.render();
    }

    fn get_configuration_warnings(&self) -> PackedStringArray {
        PackedStringArray::new()
    }
}
