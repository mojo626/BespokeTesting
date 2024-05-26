use bespoke_engine::{binding::UniformBinding, window::{SurfaceContext, WindowConfig, WindowHandler}};
use cgmath::Vector2;
use wgpu::{Color, Device, Limits, Queue, RenderPass, TextureFormat};
use winit::{dpi::{PhysicalPosition, PhysicalSize}, event::KeyEvent};



pub struct Window {
    screen_size: [f32; 2],
    screen_info_binding: UniformBinding<[f32; 4]>,
}

impl Window {
    pub fn new(device: &Device, queue: &Queue, format: TextureFormat, size: PhysicalSize<u32>) -> Self {
        let screen_size = [size.width as f32, size.height as f32];
        let screen_info_binding = UniformBinding::new(device, "Screen Size", [screen_size[0], screen_size[1], 0.0, 0.0], None);

        Self {
            screen_size,
            screen_info_binding,
        }
    }
}



impl WindowHandler for Window {
    fn resize(&mut self, _device: &Device, queue: &Queue, new_size: Vector2<u32>) {
        self.screen_size = [new_size.x as f32, new_size.y as f32];
    }

    fn render<'s: 'b, 'b>(&'s mut self, surface_ctx: &SurfaceContext, render_pass: & mut RenderPass<'b>, delta: f64) {
        println!("Hello");
    }

    fn config(&self) -> Option<WindowConfig> {
        Some(WindowConfig { background_color: Some(Color::RED), enable_post_processing: Some(false) })
    }

    fn mouse_moved(&mut self, _device: &Device, _mouse_pos: PhysicalPosition<f64>) {

    }
    
    
    fn limits() -> wgpu::Limits {
        Limits {
            max_bind_groups: 6,
            ..Default::default()
        }
    }
    
    fn other_window_event(&mut self, _device: &Device, _queue: &Queue, _event: &winit::event::WindowEvent) {
        println!("window event");
    }
    
    fn mouse_motion(&mut self, device: &Device, mouse_delta: (f64, f64)) {
        println!("mouse motion");
    }
    
    fn input_event(&mut self, device: &Device, input_event: &KeyEvent) {
        println!("input event");
    }
    
    fn touch(&mut self, device: &Device, touch: &winit::event::Touch) {
        println!("touch");
    }
    
    fn post_process_render<'a: 'b, 'c: 'b, 'b>(&'a mut self, device: &Device, queue: &Queue, render_pass: & mut wgpu::RenderPass<'b>, screen_model: &'c bespoke_engine::model::Model, surface_texture: &'c UniformBinding<bespoke_engine::texture::Texture>, depth_texture: &'c UniformBinding<bespoke_engine::texture::DepthTexture>) {
        println!("post process");
    }
}