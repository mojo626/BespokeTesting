use std::{io::Cursor, mem, ops::Range};

use bespoke_engine::{binding::{Descriptor, UniformBinding}, model::Render, shader::Shader, texture::Texture, window::{BasicVertex, SurfaceContext}};
use bytemuck::{cast_slice, NoUninit};
use image::{GenericImageView, ImageError};
use tiled::{DefaultResourceCache, Loader, ResourceCache};
use wgpu::{util::{BufferInitDescriptor, DeviceExt}, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, Buffer, BufferUsages, Color, Device, ShaderStages, TextureFormat};

use crate::window::ScreenInfo;

pub struct Tiles {
    pub width: usize,
    pub height: usize,
    pub tiles: [Vec<Tile>; 2],
    pub lights: Vec<Light>,
    pub solid: Vec<u32>,
    pub layer1_buffer: Buffer,
    pub layer2_buffer: Buffer,
    pub lights_buffer: Buffer,
    pub tiles_map_size_buffer: Buffer,
    pub solid_buffer: Buffer,
    pub tiles_bind_group: BindGroup,
    pub tiles_bind_group_layout: BindGroupLayout,
}

impl Tiles {
    fn set_solid_static(solid: &mut Vec<u32>, i: usize, is_solid: bool) {
        if Self::get_solid_static(solid, i) != is_solid {
            let num = solid[i / 32];
            let bit = i % 32;
            let shifter = 1_u32 << bit;
            let shifted = num ^ shifter;
            solid[i / 32] = shifted;
        }
    }

    fn get_solid_static(solid: &Vec<u32>, i: usize) -> bool {
        let num = solid[i / 32];
        let bit = i % 32;
        let shifter = 1_u32 << bit;
        let shifted = num & shifter;
        let shifted = shifted >> bit;
        return shifted == 1;
    }

    pub fn set_solid(&mut self, x: usize, y: usize, solid: bool) {
        if x < self.width && y < self.height {
            let i = x * self.height + y;
            Self::set_solid_static(&mut self.solid, i, solid);
        }
    }

    pub fn get_solid(&self, x: usize, y: usize) -> bool {
        if x < self.width && y < self.height {
            let i = x * self.height + y;
            return Self::get_solid_static(&self.solid, i);
        }
        return false;
    }

    pub fn set_solid_i(&mut self, i: usize, solid: bool) {
        if i < self.width*self.height {
            Self::set_solid_static(&mut self.solid, i, solid);
        }
    }

    pub fn get_solid_i(&self, i: usize) -> bool {
        if i < self.width*self.height {
            return Self::get_solid_static(&self.solid, i);
        }
        return false;
    }

    #[allow(unused)]
    pub fn test(width: usize, height: usize, device: &Device) -> Self {
        let mut layer1 = vec![];
        let mut layer2 = vec![];
        for x in 0..width {
            for y in 0..height {
                layer1.push(Tile { color: [x as f32, y as f32, 0.0, 1.0] });
                layer2.push(Tile { color: [0.0, 25.0, 0.0, 1.0] });
            }
        }
        let solid = vec![0; (layer1.len() as f32 / 32.0).ceil() as usize];
        let lights = vec![Light {color: [1.0, 1.0, 1.0, 1.0], pos: [0.0, 0.0], paddings: [0.0, 0.0]}];
        Self::new([layer1, layer2], lights, solid, width, height, device)
    }

    pub fn world(width: usize, height: usize, device: &Device) -> Self {
        let mut layer1 = vec![];
        let mut layer2 = vec![];
        let mut solid = vec![0; ((width*height) as f32 / 32.0).ceil() as usize];
        for x in 0..width {
            for y in 0..height {
                layer1.push(Tile { color: [0.0, 7.0, 0.0, 1.0] });
                layer2.push(Tile { color: [0.0, 25.0, 0.0, 1.0] });
            }
        }
        let lights = vec![Light {color: [1.0, 1.0, 1.0, 1.0], pos: [0.0, 0.0], paddings: [0.0, 0.0]}];
        Self::new([layer1, layer2], lights, solid, width, height, device)
    }

    pub fn from_image(source: &[u8], device: &Device) -> Result<Self, ImageError> {
        let image = image::load_from_memory(source)?;
        let mut layer1 = vec![];
        let mut layer2 = vec![];
        for x in 0..image.width() {
            for y in 0..image.height() {
                let pixel = image.get_pixel(x, y).0;
                layer1.push(Tile { color: [pixel[0] as f32, pixel[1] as f32, 0.0, 1.0] });
                layer2.push(Tile { color: [0.0, 25.0, 0.0, 1.0] });
            }
        }
        let lights = vec![Light {color: [1.0, 1.0, 1.0, 1.0], pos: [0.0, 0.0], paddings: [0.0, 0.0]}];
        let solid = vec![0; (layer1.len() as f32 / 32.0).ceil() as usize];
        Ok(Self::new([layer1, layer2], lights, solid, image.width() as usize, image.height() as usize, device))
    }

    pub fn from_tmx(source: &[u8], default: [f32; 2], device: &Device) -> Self {
        let res_reader = MyReader {
            src: source,
        };
        let mut loader = Loader::with_cache_and_reader(DefaultResourceCache::default(), res_reader);
        let map = loader.load_tmx_map("doesn't matter").unwrap();
        let layer1_src = map.layers().filter(|layer| layer.id() == 1).last().unwrap().as_tile_layer().unwrap();
        let layer2_src = map.layers().filter(|layer| layer.id() == 2).last().unwrap().as_tile_layer().unwrap();
        let mut layer1 = vec![];
        let mut layer2 = vec![];
        for x in 0..map.width {
            for y in 0..map.height {
                if let Some(tile) = layer1_src.get_tile(x as i32, y as i32) {
                    let width = tile.get_tileset().columns;
                    let tile_i = tile.id();
                    let tile_x = tile_i % width;
                    let tile_y = tile_i / width;
                    layer1.push(Tile { color: [tile_x as f32, tile_y as f32, 0.0, 1.0] });
                } else {
                    layer1.push(Tile { color: [default[0], default[1], 0.0, 1.0] });
                }
                if let Some(tile) = layer2_src.get_tile(x as i32, y as i32) {
                    let width = tile.get_tileset().columns;
                    let tile_i = tile.id();
                    let tile_x = tile_i % width;
                    let tile_y = tile_i / width;
                    layer2.push(Tile { color: [tile_x as f32, tile_y as f32, 0.0, 1.0] });
                } else {
                    layer2.push(Tile { color: [default[0], default[1], 0.0, 1.0] });
                }
            }
        }
        let lights = vec![Light {color: [1.0, 1.0, 1.0, 1.0], pos: [0.0, 0.0], paddings: [0.0, 0.0]}];
        let solid = vec![0; (layer1.len() as f32 / 32.0).ceil() as usize];
        Self::new([layer1, layer2], lights, solid, map.width as usize, map.height as usize, device)
    }

    fn new(tiles: [Vec<Tile>; 2], lights: Vec<Light>, solid: Vec<u32>, width: usize, height: usize, device: &Device) -> Self {
        let layer1_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Tiles Layer 1 Buffer"),
            contents: cast_slice(&tiles[0]),
            usage: BufferUsages::STORAGE,
        });
        let layer2_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Tiles Layer 2 Buffer"),
            contents: cast_slice(&tiles[1]),
            usage: BufferUsages::STORAGE,
        });
        let lights_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Lights Buffer"),
            contents: cast_slice(&lights),
            usage: BufferUsages::STORAGE,
        });
        let tiles_map_size_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Tile Map Size Buffer"),
            contents: cast_slice(&[width as u32, height as u32]),
            usage: BufferUsages::UNIFORM,
        });
        let solid_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Solid Buffer"),
            contents: cast_slice(&solid),
            usage: BufferUsages::STORAGE,
        });
        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Tiles Buffer Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::all(),
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::all(),
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::all(),
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::all(),
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::all(),
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
            ]
        });
        let tiles_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Tiles Buffer Bind Group"),
            layout: &layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: layer1_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: layer2_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: tiles_map_size_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: lights_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: solid_buffer.as_entire_binding(),
                }
            ]
        });
        Self { width, height, tiles, lights, solid, layer1_buffer, layer2_buffer, lights_buffer, tiles_bind_group, tiles_bind_group_layout: layout, tiles_map_size_buffer, solid_buffer }
    }

    pub fn recreate_tiles(&mut self, layers: Range<usize>, device: &Device) {
        if layers.contains(&0) {
            self.layer1_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Tiles Layer 1 Buffer"),
                contents: cast_slice(&self.tiles[0]),
                usage: BufferUsages::STORAGE,
            });
        }
        if layers.contains(&1) {
            self.layer2_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Tiles Layer 2 Buffer"),
                contents: cast_slice(&self.tiles[1]),
                usage: BufferUsages::STORAGE,
            });
        }
        let tiles_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Tiles Buffer Bind Group"),
            layout: &self.tiles_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: self.layer1_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: self.layer2_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: self.tiles_map_size_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: self.lights_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: self.solid_buffer.as_entire_binding(),
                }
            ]
        });
        self.tiles_bind_group = tiles_bind_group;
    }

    pub fn recreate_lights(&mut self, device: &Device) {
        let lights_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Lights Buffer"),
            contents: cast_slice(&self.lights),
            usage: BufferUsages::STORAGE,
        });
        let tiles_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Tiles Buffer Bind Group"),
            layout: &self.tiles_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: self.layer1_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: self.layer2_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: self.tiles_map_size_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: lights_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: self.solid_buffer.as_entire_binding(),
                },
            ]
        });
        self.lights_buffer = lights_buffer;
        self.tiles_bind_group = tiles_bind_group;
    }

    pub fn recreate_solid(&mut self, device: &Device) {
        let solid_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Solid Buffer"),
            contents: cast_slice(&self.solid),
            usage: BufferUsages::STORAGE,
        });
        let tiles_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Tiles Buffer Bind Group"),
            layout: &self.tiles_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: self.layer1_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: self.layer2_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: self.tiles_map_size_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: self.lights_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: solid_buffer.as_entire_binding(),
                },
            ]
        });
        self.solid_buffer = solid_buffer;
        self.tiles_bind_group = tiles_bind_group;
    }

    pub fn render(&mut self, surface_context: &SurfaceContext, format: TextureFormat, screen_info_binding: &UniformBinding<ScreenInfo>) {
        let temp_texture = Texture::blank_texture(&surface_context.device, surface_context.config.width, surface_context.config.height, surface_context.config.format);
        let mut encoder = surface_context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Shadow Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &temp_texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(Color::RED),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                timestamp_writes: None,
                occlusion_query_set: None,
                depth_stencil_attachment: None,
            });
            let tiles_shader = Shader::new(include_str!("tiles.wgsl"), &surface_context.device, format, vec![], &[BasicVertex::desc()], None);
            let tile_set_texture = UniformBinding::new(&surface_context.device, "Tile Set", Texture::from_image(&surface_context.device, &surface_context.queue, img, label, format, sample_type, filter_mode), None);
            render_pass.set_pipeline(&tiles_shader.pipeline);
            render_pass.set_bind_group(0, &self.tiles_bind_group, &[]);
            render_pass.set_bind_group(2, &screen_info_binding.binding, &[]);
            render_pass.set_bind_group(3, &self.textures.binding, &[]);
            render_pass.set_bind_group(4, &self.sprites.binding, &[]);
            surface_context.screen_model.render(&mut render_pass);
        }
    }
}

#[repr(C)]
#[derive(NoUninit, Clone, Copy)]
pub struct Tile {
    pub color: [f32; 4],
}

//A VEC3 IS NOT ALLIGNED

#[repr(C)]
#[derive(NoUninit, Clone, Copy)]
pub struct Light {
    pub color: [f32; 4],
    pub pos: [f32; 2],
    pub paddings: [f32; 2],
}

struct MyReader<'b> {
    src: &'b [u8],
}

impl <'b> tiled::ResourceReader for MyReader<'b> {
    type Resource = Cursor<&'b [u8]>;
    type Error = std::io::Error;

    fn read_from(&mut self, _: &std::path::Path) -> std::result::Result<Self::Resource, Self::Error> {
        Ok(Cursor::new(&self.src))
    }
}

mod test {
    #[test]
    fn test_bits() {
        let num = 0_u32;
        let bit = 3_u32;
        let shifter = 1_u32 << bit;
        let shifted = num ^ shifter;
        assert_eq!(shifted, 8_u32);
        let shifter = 1_u32 << bit;
        println!("{shifter:b}");
        let shifted = shifted & shifter;
        println!("{shifted:b}");
        let shifted = shifted >> bit;
        println!("{shifted:b}");
        assert_eq!(shifted, 1_u32);
    }
}