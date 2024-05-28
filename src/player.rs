use cgmath::{Vector2, Vector3};
use wgpu::{Device, RenderPass};
use winit::keyboard::KeyCode;

use crate::{physics::boxCollider::BoxCollider, shaders::ShaderManager, sprite::Sprite};



pub struct Player {
    pub pos: Vector2<f32>,
    vel: Vector2<f32>,
    sprite: Sprite,
    collider: BoxCollider,
    touching_ground: bool,
}

impl Player {
    pub fn new(pos: Vector2<f32>, sprite: Sprite) -> Self {
        let collider = BoxCollider::new(pos, Vector2::new(50.0, 50.0));
        let vel = Vector2::new(0.0, 0.0);
        let touching_ground = false;

        Self {
            pos,
            sprite,
            collider,
            vel,
            touching_ground,
        }
    }

    pub fn render<'s: 'b + 'm, 'b,'m: 'b>(&'s mut self, render_pass: &mut RenderPass<'b>, shaderMan: &'m ShaderManager) {
        self.sprite.render(render_pass, shaderMan);
    }

    pub fn handle_input(&mut self, keys_down: &Vec<KeyCode>, device: &Device, delta: f32, terrain: &Vec<BoxCollider>) {
        let speed = 0.2;
        let previous_y = self.pos.y;
        let previous_x = self.pos.x;
        let jump_force = -8.0;

        let mut move_amount = Vector2::new(0.0, 0.0);

        if keys_down.contains(&KeyCode::KeyA) {
            move_amount -= Vector2::new(1.0, 0.0) * speed * delta;
        }
        if keys_down.contains(&KeyCode::KeyD) {
            move_amount += Vector2::new(1.0, 0.0) * speed * delta;
        }
        if keys_down.contains(&KeyCode::KeyW) && self.touching_ground {
            self.vel.y = jump_force;
        }

        //gravity constant
        self.vel.y += 0.01 * delta;

        move_amount.y -= self.vel.y;

        let mut collided_y = false;
        let mut collided_x = false;


        self.touching_ground = false;
        for other_coll in terrain {
            self.collider.pos = Vector2::new(self.pos.x, self.pos.y + move_amount.y);
            if self.collider.CheckCollision(&other_coll)
            {
                self.vel.y = 0.0;
                self.touching_ground = true;
                collided_y = true;
            }
            self.collider.pos = Vector2::new(self.pos.x + move_amount.x, self.pos.y);
            if self.collider.CheckCollision(&other_coll)
            {
                collided_x = true;
            }
        }

        if (!collided_y)
        {
            self.pos.y += move_amount.y;
        }
        if (!collided_x)
        {
            self.pos.x += move_amount.x;
        }

        self.sprite.set_position(Vector3::new(0.0, self.pos.y, self.pos.x), device);
    }
}