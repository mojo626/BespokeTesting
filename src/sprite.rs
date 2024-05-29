use bespoke_engine::{binding::{Descriptor, UniformBinding}, camera::{self, Camera}, model::Render, shader::{Shader, ShaderConfig}, texture::Texture};
use cgmath::{Quaternion, Rotation, Vector3};
use wgpu::{core::device, Device, Queue, RenderPass, TextureFormat};

use crate::{billboard::Billboard, instance::Instance, load_resource, shaders::ShaderManager, window::Vertex};



pub struct Sprite {
    sprite_image: UniformBinding<Texture>,
    billboard: Billboard,
    shader: String,
}


impl Sprite {
    pub fn new(path: &str, device: &Device, queue: &Queue, camera_binding: & UniformBinding<[[f32; 4]; 4]>, format: TextureFormat, scale: f32, position: Vector3<f32>, shader: String) -> Self {
        let sprite_image = UniformBinding::new(device, "sprite", Texture::from_bytes(device, queue, &load_resource(path).unwrap(), "image", Some(wgpu::FilterMode::Nearest)).unwrap(), None);
        let mut sprite_dim = sprite_image.value.normalized_dimensions();
        sprite_dim = (1.0, sprite_dim.1/sprite_dim.0);
        let rotation = Quaternion::look_at(Vector3::new(-1.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0));
        let billboard = Billboard::new(sprite_dim.0 * scale, sprite_dim.1 * scale, 1.0, position, rotation, device);
        Self {
            sprite_image,
            shader,
            billboard,
        }
    }

    pub fn render<'b, 's: 'b>(&'s mut self, render_pass: & mut RenderPass<'b>, shader_man: * mut ShaderManager) {
        unsafe {
            (*shader_man).bind_shader(self.shader.clone(), render_pass);
        }
        render_pass.set_bind_group(1, &self.sprite_image.binding, &[]);

        self.billboard.render(render_pass);
    }

    pub fn set_position(&mut self, new_pos: Vector3<f32>, device: &Device) {
        self.billboard.set_position(new_pos, device);
    }
}