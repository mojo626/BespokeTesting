use std::time::{SystemTime, UNIX_EPOCH};

use bespoke_engine::{billboard::Billboard, binding::{create_layout, Descriptor, UniformBinding}, camera::Camera, instance::Instance, model::{Render, ToRaw}, shader::{self, Shader, ShaderConfig}, texture::Texture, window::{SurfaceContext, WindowConfig, WindowHandler}};
use bytemuck::{bytes_of, NoUninit, Pod, Zeroable};
use cgmath::{Quaternion, Rotation, Vector2, Vector3};
use tiled::Tile;
use wgpu::{Color, Device, Limits, PipelineLayout, Queue, RenderPass, TextureFormat};
use winit::{dpi::{PhysicalPosition, PhysicalSize}, event::KeyEvent, keyboard::{KeyCode, PhysicalKey::Code}};

use crate::{load_resource, player::Player, shaders::ShaderManager, sprite::{self, Sprite}, TilesetManager::TilesetManager};

pub struct Window {
    screen_size: [f32; 2],
    screen_info_binding: UniformBinding<[f32; 4]>,
    start_time: u128,
    camera: Camera,
    camera_binding: UniformBinding<[[f32; 4]; 4]>,
    keys_down: Vec<KeyCode>,
    sprite: Sprite,
    sprite2: Sprite,
    shaderMan: ShaderManager,
    tileset_sprite: Sprite,
    player: Player,
    tileset_man: TilesetManager,
}

#[repr(C)]
#[derive(Pod, Clone, Copy, Zeroable)]
pub struct ScreenInfo {
    screen_size: [f32; 2],
    scroll: [f32; 2],
    mouse_pos: [f32; 2],
    time: f32,
    tiles_on_screen_size: f32,
    tile_set_size: [f32; 2],
}

impl ScreenInfo {
    fn new(screen_size: Vector2<f32>, scroll: Vector2<f32>, mouse_pos: Vector2<f32>, time: f32, tiles_on_screen_size: f32, tile_set_size: [f32; 2]) -> Self {
        Self {
            screen_size: screen_size.into(),
            scroll: scroll.into(),
            mouse_pos: mouse_pos.into(),
            time,
            tiles_on_screen_size,
            tile_set_size,

        }
    }
}

impl Window {
    pub fn new(device: &Device, queue: &Queue, format: TextureFormat, size: PhysicalSize<u32>) -> Self {
        let screen_size = [size.width as f32, size.height as f32];
        let camera = Camera {
            eye: Vector3::new(-180.0, 0.0, 0.0),
            // eye: Vector3::new(0.0, 0.0, 0.0),
            aspect: screen_size[0] / screen_size[1],
            fovy: 70.0,
            znear: 0.1,
            zfar: 100.0,
            ground: 0.0,
            sky: 0.0,
        };

        let camera_binding = UniformBinding::new(device, "Camera", camera.build_view_projection_matrix_raw(), None);
        let screen_info_binding = UniformBinding::new(device, "Screen Size", [screen_size[0], screen_size[1], 0.0, 0.0], None);
        let mut shaderMan = ShaderManager::new();
        let billboard_shader = Shader::new(include_str!("billboard.wgsl"), device, format, vec![&camera_binding.layout, &create_layout::<Texture>(device)], &[Vertex::desc(), Instance::desc()], Some(ShaderConfig {background: Some(false), ..Default::default()}));
        shaderMan.shaders.insert("billboard".into(), billboard_shader);
        
        let start_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();

        

        let sprite = Sprite::new(r"res\BGFront.png", device, queue, &camera_binding, format, 1500.0, Vector3::new(150.0, 0.0, 0.0), "billboard".into());

        let sprite2 = Sprite::new(r"res\BGBack.png", device, queue, &camera_binding, format, 1500.0, Vector3::new(300.0, 0.0, 0.0), "billboard".into());

        
        let tileset_man = TilesetManager::new("src/res/map.json", 800);
        let tileset_sprite = Sprite::new(r"res\output.png", device, queue, &camera_binding, format, 800.0, Vector3::new(0.0, 0.0, 0.0), "billboard".into());

        let player_sprite = Sprite::new(r"res\player.png", device, queue, &camera_binding, format, 50.0, Vector3::new(0.0, 0.0, 0.0), "billboard".into());
        let player = Player::new(Vector2::new(100.0, 0.0), player_sprite);

        Self {
            screen_size,
            screen_info_binding,
            start_time,
            camera,
            camera_binding,
            keys_down: vec![],
            sprite,
            sprite2,
            shaderMan,
            tileset_sprite,
            player,
            tileset_man,
        }
    }
}

#[repr(C)]
#[derive(NoUninit, Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_pos: [f32; 2],
    pub normal: [f32; 3],
}

impl Vertex {
    #[allow(dead_code)]
    pub fn pos(&self) -> Vector3<f32> {
        return Vector3::new(self.position[0], self.position[1], self.position[2]);
    }
}

impl Descriptor for Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

impl ToRaw for Vertex {
    fn to_raw(&self) -> Vec<u8> {
        bytes_of(self).to_vec()
    }
}


impl WindowHandler for Window {
    fn resize(&mut self, device: &Device, queue: &Queue, new_size: Vector2<u32>) {
        self.screen_size = [new_size.x as f32, new_size.y as f32];
    }

    fn render<'s: 'c, 'c>(&'s mut self, surface_ctx: &SurfaceContext, render_pass: & mut RenderPass<'c>, delta: f64) {
        let speed = 0.2 * delta as f32;

        self.player.handle_input(&self.keys_down, &surface_ctx.device, delta as f32, &self.tileset_man.colliders);
        
        self.camera.eye.y = self.player.pos.y;
        self.camera.eye.z = self.player.pos.x;

        
        self.camera_binding.set_data(&surface_ctx.device, self.camera.build_view_projection_matrix_raw());


        let time = (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()-self.start_time) as f32 / 1000.0;
        self.screen_info_binding.set_data(&surface_ctx.device, [self.screen_size[0], self.screen_size[1], time, 0.0]);

        render_pass.set_bind_group(0, &self.camera_binding.binding, &[]);

        //rust borrow checker :(
        let man_ref1: *mut ShaderManager = &mut self.shaderMan;
        
        unsafe {
            (*man_ref1).bind_active_shader(render_pass);
        }
        

        self.sprite2.render(render_pass, man_ref1);

        self.sprite.render(render_pass, man_ref1);

        self.tileset_sprite.render(render_pass, man_ref1);

        self.player.render(render_pass, man_ref1);

    }

    fn config(&self) -> Option<WindowConfig> {
        Some(WindowConfig { background_color: Some(Color::BLUE), enable_post_processing: Some(false) })
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
    }
    
    fn mouse_motion(&mut self, device: &Device, mouse_delta: (f64, f64)) {
    }
    
    fn input_event(&mut self, device: &Device, input_event: &KeyEvent) {
        if let Code(code) = input_event.physical_key {
            if input_event.state.is_pressed() {
                if !self.keys_down.contains(&code) {
                    self.keys_down.push(code);
                }
            } else {
                if let Some(i) = self.keys_down.iter().position(|x| x == &code) {
                    self.keys_down.remove(i);
                }
            }
        }
    }
    
    fn touch(&mut self, device: &Device, touch: &winit::event::Touch) {
        println!("touch");
    }
    
    fn post_process_render<'a: 'b, 'c: 'b, 'b>(&'a mut self, device: &Device, queue: &Queue, render_pass: & mut wgpu::RenderPass<'b>, screen_model: &'c bespoke_engine::model::Model, surface_texture: &'c UniformBinding<bespoke_engine::texture::Texture>, depth_texture: &'c UniformBinding<bespoke_engine::texture::DepthTexture>) {
        println!("post process");
    }
}