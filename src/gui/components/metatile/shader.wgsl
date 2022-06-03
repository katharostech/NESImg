struct Tile {
    [[align(16)]] tex_idx: u32;
    uv_start: vec2<f32>;
    uv_size: vec2<f32>;
};

struct Metatile {
    tiles: array<Tile, 4>;
};

[[group(0), binding(0)]]
var<uniform> metatile: Metatile;

struct VertexOut {
    [[builtin(position)]] pos: vec4<f32>;
    [[location(0)]] tex_idx: u32;
    [[location(1)]] uv: vec2<f32>;
};

var<private> v_positions: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(-1.0, 0.0),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(0.0, 0.0),
    vec2<f32>(-1.0, 0.0),
);

// Notice how the uvs don't quite go to zero or one. That is to keep the sampler from overshooting
// slightly and sampling the color from the next tile, resulting in an unslightly seam between
// tiles.
var<private> v_uvs: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(0.01, 0.01),
    vec2<f32>(0.99, 0.01),
    vec2<f32>(0.01, 0.99),
    vec2<f32>(0.99, 0.01),
    vec2<f32>(0.99, 0.99),
    vec2<f32>(0.01, 0.99),
);

[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] vertex_idx: u32) -> VertexOut {
    var out: VertexOut;
    let vertex_idx_in_square = vertex_idx % 6u;

    let offset_x = (vertex_idx / 6u) % 2u;
    let offset_y = vertex_idx / 6u / 2u;
    let offset = vec2<f32>(f32(offset_x), -f32(offset_y));

    let tile = metatile.tiles[vertex_idx / 6u];

    out.pos = vec4<f32>(v_positions[vertex_idx_in_square] + offset, 0.0, 1.0);
    out.uv = tile.uv_start + tile.uv_size * v_uvs[vertex_idx_in_square];
    out.tex_idx = tile.tex_idx;
    return out;
}

[[group(0), binding(1)]]
var dummy_texture: texture_2d<f32>;
[[group(0), binding(2)]]
var texture_0: texture_2d<f32>;
[[group(0), binding(3)]]
var texture_1: texture_2d<f32>;
[[group(0), binding(4)]]
var texture_2: texture_2d<f32>;
[[group(0), binding(5)]]
var texture_3: texture_2d<f32>;

[[group(0), binding(6)]]
var tex_sampler: sampler;

[[stage(fragment)]]
fn fs_main(in: VertexOut) -> [[location(0)]] vec4<f32> {
    var out: vec4<f32>;

    if (in.tex_idx == 0u) {
        out = textureSampleLevel(dummy_texture, tex_sampler, in.uv, 0.0);
    } else if (in.tex_idx == 1u) {
        out = textureSampleLevel(texture_0, tex_sampler, in.uv, 0.0);
    } else if (in.tex_idx == 2u) {
        out = textureSampleLevel(texture_1, tex_sampler, in.uv, 0.0);
    } else if (in.tex_idx == 3u) {
        out = textureSampleLevel(texture_2, tex_sampler, in.uv, 0.0);
    } else if (in.tex_idx == 4u) {
        out = textureSampleLevel(texture_3, tex_sampler, in.uv, 0.0);
    }

    return out;
}