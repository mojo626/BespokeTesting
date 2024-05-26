use bespoke_engine::{binding::{Descriptor, UniformBinding}, camera::{self, Camera}, model::Render, shader::{Shader, ShaderConfig}, texture::Texture};
use cgmath::{Quaternion, Rotation, Vector3};
use wgpu::{core::device, Device, Queue, RenderPass, TextureFormat};

use crate::{billboard::Billboard, instance::Instance, load_resource, window::Vertex};



pub struct Sprite {
    sprite_image: UniformBinding<Texture>,
    billboard: Billboard,
    billboard_shader: Shader,
}


impl Sprite {
    pub fn new(path: &str, camera: &Camera, device: &Device, queue: &Queue, camera_binding: & UniformBinding<[[f32; 4]; 4]>, format: TextureFormat, scale: f32, position: Vector3<f32>) -> Self {
        let sprite_image = UniformBinding::new(device, "sprite", Texture::from_bytes(device, queue, &load_resource(path).unwrap(), "image", Some(wgpu::FilterMode::Nearest)).unwrap(), None);
        let sprite_dim = sprite_image.value.normalized_dimensions();
        let rotation = Quaternion::look_at((camera.eye - position), Vector3::new(0.0, 1.0, 0.0));
        let billboard = Billboard::new(sprite_dim.0 * scale, sprite_dim.1 * scale, 1.0, position, rotation, device);
        let billboard_shader = Shader::new(include_str!("billboard.wgsl"), device, format, vec![&camera_binding.layout, &sprite_image.layout], &[Vertex::desc(), Instance::desc()], Some(ShaderConfig {background: Some(false), ..Default::default()}));

        Self {
            sprite_image,
            billboard_shader,
            billboard,
        }
    }

    pub fn render<'b, 's: 'b>(&'s mut self, render_pass: & mut RenderPass<'b>, camera_binding: &'b UniformBinding<[[f32; 4]; 4]>) {
        self.billboard_shader.bind(render_pass);

        render_pass.set_bind_group(0, &camera_binding.binding, &[]);
        render_pass.set_bind_group(1, &self.sprite_image.binding, &[]);

        self.billboard.render(render_pass);
    }
}