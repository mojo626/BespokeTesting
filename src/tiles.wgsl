struct Tile {
    color: vec4f,
};

struct ScreenInfo {
    screen_size: vec2f,
    scroll: vec2f,
    mouse_pos: vec2f,
    time: f32,
    tiles_on_screen_size: f32,
    tile_set_size: vec2f,
    // padding: vec2f,
    // player: Player,
};

struct Player {
    pos: vec2f,
};

struct Light {
    color: vec4f,
    pos: vec2f,
}

struct Sprite {
    pos: vec2f,
    screen_size: vec2f,
    tex_coords: vec2f,
    texture_size: vec2f,
}

const PLAYER_SIZE: f32 = 1.0;

@group(0) @binding(0)
var<storage, read> tiles_layer1: array<vec4f>;
@group(0) @binding(1)
var<storage, read> tiles_layer2: array<vec4f>;
@group(0) @binding(2)
var<uniform> tile_map_size: vec2<u32>;
@group(0) @binding(3)
var<storage, read> lights: array<Light>;
@group(0) @binding(4)
var<storage, read> solids: array<u32>;

@group(1) @binding(0)
var t_shadows: texture_2d<f32>;
@group(1) @binding(1)
var s_shadows: sampler;

@group(2) @binding(0)
var<uniform> screen_info: ScreenInfo;

@group(3) @binding(0)
var t_tile_set: texture_2d<f32>;
@group(3) @binding(1)
var s_tile_set: sampler;
@group(3) @binding(2)
var t_sprite_set: texture_2d<f32>;
@group(3) @binding(3)
var s_sprite_set: sampler;

@group(4) @binding(0)
var<storage, read> sprites: array<Sprite>;
@group(4) @binding(1)
var<uniform> sprite_set_size: vec2f;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 1.0);
    out.tex_coords = model.tex_coords;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tiles_pos = in.tex_coords*screen_info.screen_size/screen_info.tiles_on_screen_size + screen_info.scroll;
    var color = foreground_color(tiles_pos, in);
    if color.w == 1.0 {
        return color;
    }
    for (var i: i32 = 0; i < i32(arrayLength(&sprites)); i++) {
        let sprite = sprites[i];
        if (tiles_pos.x >= sprite.pos.x && tiles_pos.y >= sprite.pos.y && tiles_pos.x < sprite.pos.x+sprite.screen_size.x && tiles_pos.y < sprite.pos.y+sprite.screen_size.y) {
            let tex_coord = (tiles_pos-sprite.pos)/sprite.screen_size/sprite_set_size*sprite.texture_size + sprite.tex_coords/sprite_set_size;
            let sprite_color = textureSample(t_sprite_set, s_sprite_set, tex_coord);
            color = alpha_composite(sprite_color, color);
            if color.w == 1.0 {
                return color;
            }
        }
    }
    return alpha_composite(background_color(tiles_pos, in), color);
}

fn foreground_color(tiles_pos: vec2f, in: VertexOutput) -> vec4f {
    let mouse_pos = screen_info.mouse_pos/screen_info.tiles_on_screen_size + screen_info.scroll;
    let mod_x = modf(tiles_pos.x);
    let mod_y = modf(tiles_pos.y);
    let x = mod_x.whole;
    let y = mod_y.whole;
    let x_fract = mod_x.fract;
    let y_fract = mod_y.fract;
    let mouse_i = i32(u32(modf(mouse_pos.x).whole) * tile_map_size.y + u32(modf(mouse_pos.y).whole));
    if (tiles_pos.x < f32(tile_map_size.x) && tiles_pos.x >= 0.0 && tiles_pos.y < f32(tile_map_size.y) && tiles_pos.y >= 0.0) {
        let i = i32(u32(x) * tile_map_size.y + u32(y));
        let tile = tiles_layer2[i];
        var color = textureSample(t_tile_set, s_tile_set, (tile.xy + vec2f(x_fract, y_fract))/screen_info.tile_set_size);
        var processed_color = color;
        let light = calc_lights(tiles_pos);
        processed_color = vec4f(processed_color.xyz*light, processed_color.w);
        if mouse_i == i {
            processed_color = vec4f(processed_color.xyz*1.5, processed_color.w);
        }
        return processed_color;
    }
    return vec4f(0.0, 0.0, 0.0, 0.0);
}

fn background_color(tiles_pos: vec2f, in: VertexOutput) -> vec4f {
    let mouse_pos = screen_info.mouse_pos/screen_info.tiles_on_screen_size + screen_info.scroll;
    let mod_x = modf(tiles_pos.x);
    let mod_y = modf(tiles_pos.y);
    let x = mod_x.whole;
    let y = mod_y.whole;
    let x_fract = mod_x.fract;
    let y_fract = mod_y.fract;
    let mouse_i = i32(u32(modf(mouse_pos.x).whole) * tile_map_size.y + u32(modf(mouse_pos.y).whole));
    if (tiles_pos.x < f32(tile_map_size.x) && tiles_pos.x >= 0.0 && tiles_pos.y < f32(tile_map_size.y) && tiles_pos.y >= 0.0) {
        let i = i32(u32(x) * tile_map_size.y + u32(y));
        let tile = tiles_layer1[i];
        var color = textureSample(t_tile_set, s_tile_set, (tile.xy + vec2f(x_fract, y_fract))/screen_info.tile_set_size);
        // var color = vec4f((tile.xy + vec2f(x_fract, y_fract))/screen_info.tile_set_size, 0.0, 1.0);
        // color.w = tile.w*255.0;
        var processed_color = color;
        let light_color = calc_lights(tiles_pos);
        let raycast_light = textureSample(t_shadows, s_shadows, in.tex_coords);
        let light = vec3f(min(max(light_color.x, raycast_light.x), 1.0), min(max(light_color.y, raycast_light.y), 1.0), min(max(light_color.z, raycast_light.z), 1.0));
        processed_color = processed_color + vec4f(0.3882352941, 0.6235294118, 1.0, 1.0)*(1.0-processed_color.w);
        processed_color = vec4f(processed_color.xyz*light, processed_color.w);
        if mouse_i == i {
            processed_color = vec4f(processed_color.xyz*1.5, processed_color.w);
        }
        return processed_color;
    }
    let light_color = calc_lights(tiles_pos);
    let raycast_light = textureSample(t_shadows, s_shadows, in.tex_coords);
    let light = vec3f(min(max(light_color.x, raycast_light.x), 1.0), min(max(light_color.y, raycast_light.y), 1.0), min(max(light_color.z, raycast_light.z), 1.0));
    return vec4f(vec3f(0.3882352941, 0.6235294118, 1.0)*light, 1.0);
}

const AMBIENT_LIGHT: f32 = 1.0;

//further optimizations I'm not adding:
//cull lights that have no effect on the viewing window (dist to edge > effect radius)
//return when the light is higher than the max
fn calc_lights(point: vec2f) -> vec3f {
    var comp = vec3f(AMBIENT_LIGHT, AMBIENT_LIGHT, AMBIENT_LIGHT);
    for (var i: i32 = 0; i < i32(arrayLength(&lights)); i++) {
        if min_val3(comp) >= 1.0 {
            return comp;
        }
        let light = lights[i];
        comp += (max(1-distance(light.pos, point)/light.color.w, 0.0))*light.color.xyz;
    }
    return comp;
}

fn alpha_composite(c1: vec4f, c2: vec4f) -> vec4f
{
    var opa1 = c1.w;
    var opa2 = c2.w;
    if opa1 == 0.0 && opa2 == 1.0 {
        return c2;
    }
    if opa2 == 0.0 && opa1 == 1.0 {
        return c1;
    }
    var ar = opa1 + opa2 - (opa1 * opa2);
    var asr = opa2 / ar;
    var a1 = 1.0 - asr;
    var a2 = asr * (1.0 - opa1);
    var ab = asr * opa1;
    var r = c1.x * a1 + c2.x* a2 + c2.x * ab;
    var g = c1.y * a1 + c2.y * a2 + c2.y * ab;
    var b = c1.z * a1 + c2.z * a2 + c2.z * ab;
    return vec4f(r, g, b, ar);
}

fn min_val4(val: vec4f) -> f32 {
    return min(val.x, min(val.y, min(val.z, val.w)));
}

fn min_val3(val: vec3f) -> f32 {
    return min(val.x, min(val.y, val.z));
}