use cgmath::{Vector2, Vector3};
use wgpu::{Device, RenderPass};
use winit::keyboard::KeyCode;

use crate::{physics::boxCollider::BoxCollider, shaders::ShaderManager, sprite::Sprite};



pub struct Player {
    pub pos: Vector2<f32>,
    vel: Vector2<f32>,
    sprite: Sprite,
    collider: BoxCollider,
}

impl Player {
    pub fn new(pos: Vector2<f32>, sprite: Sprite) -> Self {
        let collider = BoxCollider::new(pos, Vector2::new(50.0, 50.0));
        let vel = Vector2::new(0.0, 0.0);

        Self {
            pos,
            sprite,
            collider,
            vel,
        }
    }

    pub fn render<'s: 'b + 'm, 'b,'m: 'b>(&'s mut self, render_pass: &mut RenderPass<'b>, shaderMan: &'m ShaderManager) {
        self.sprite.render(render_pass, shaderMan);
    }

    pub fn handle_input(&mut self, keys_down: &Vec<KeyCode>, device: &Device, delta: f32, terrain: &Vec<BoxCollider>) {
        let speed = 0.2;
        let previous_y = self.pos.y;
        let jump_force = -0.5;

        if keys_down.contains(&KeyCode::KeyA) {
            self.pos -= Vector2::new(1.0, 0.0) * speed * delta;
        }
        if keys_down.contains(&KeyCode::KeyD) {
            self.pos += Vector2::new(1.0, 0.0) * speed * delta;
        }
        if keys_down.contains(&KeyCode::KeyW) {
            self.vel.y = jump_force;
        }

        //gravity constant
        self.vel.y += 0.001;

        self.pos.y -= self.vel.y;

        self.collider.pos = self.pos;

        for other_coll in terrain {
            if self.collider.CheckCollision(&other_coll)
            {
                self.pos.y = previous_y;
                self.vel.y = 0.0;
            }
        }

        

        self.sprite.set_position(Vector3::new(0.0, self.pos.y, self.pos.x), device);
    }
}