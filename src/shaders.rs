use std::collections::HashMap;

use bespoke_engine::shader::Shader;
use wgpu::RenderPass;



pub struct ShaderManager {
    pub shaders: HashMap<String, Shader>,
    pub active_shader: String,
}

impl ShaderManager {
    pub fn new() -> Self {
        let shaders = HashMap::new();

        Self {
            shaders,
            active_shader: "👻".into(),
        }
    }

    pub fn bind_shader<'b, 's: 'b>(&'s self, id: String, render_pass: & mut RenderPass<'b>) {
        if self.active_shader == id {
            return;
        }
        if let Some(shader) = self.shaders.get(&id) {
            // self.active_shader = id;
            shader.bind(render_pass);
        }
    }
}