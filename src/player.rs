use cgmath::{Vector2, Vector3};
use wgpu::{Device, RenderPass};
use winit::keyboard::KeyCode;

use crate::{physics::boxCollider::BoxCollider, shaders::ShaderManager, sprite::Sprite};



pub struct Player {
    pub pos: Vector2<f32>,
    sprite: Sprite,
    collider: BoxCollider,
}

impl Player {
    pub fn new(pos: Vector2<f32>, sprite: Sprite) -> Self {
        let collider = BoxCollider::new(pos, Vector2::new(50.0, 50.0));

        Self {
            pos,
            sprite,
            collider,
        }
    }

    pub fn render<'s: 'b + 'm, 'b,'m: 'b>(&'s mut self, render_pass: &mut RenderPass<'b>, shaderMan: &'m ShaderManager) {
        self.sprite.render(render_pass, shaderMan);
    }

    pub fn handle_input(&mut self, keys_down: &Vec<KeyCode>, device: &Device, delta: f32) {
        let speed = 0.2;
        if keys_down.contains(&KeyCode::KeyW) {
            self.pos += Vector2::new(0.0, 1.0) * speed * delta;
        }
        if keys_down.contains(&KeyCode::KeyS) {
            self.pos -= Vector2::new(0.0, 1.0) * speed * delta;
        }
        if keys_down.contains(&KeyCode::KeyA) {
            self.pos -= Vector2::new(1.0, 0.0) * speed * delta;
        }
        if keys_down.contains(&KeyCode::KeyD) {
            self.pos += Vector2::new(1.0, 0.0) * speed * delta;
        }

        self.collider.pos = self.pos;

        let other_coll = BoxCollider::new(Vector2::new(0.0, 0.0), Vector2::new(50.0, 50.0));

        if (self.collider.CheckCollision(&other_coll))
        {
            println!("Colliding!");
        }

        self.sprite.set_position(Vector3::new(0.0, self.pos.y, self.pos.x), device);
    }
}