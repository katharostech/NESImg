use std::sync::Arc;

use eframe::egui;
use egui::{mutex::Mutex, Color32, Rect, Vec2};

use egui_extras::RetainedImage;
use glow::HasContext;

use crate::{
    app::ImagePalletData,
    globals::{NES_PALLET_SHADER_CONST, TILE_SIZE, TILE_SIZE_INT},
};

pub struct NesImageViewer<'a> {
    id: egui::Id,
    image: &'a RetainedImage,
    current_pallet: u8,
    pallet: &'a mut ImagePalletData,
}

#[derive(Clone, Copy)]
struct ViewerState {
    zoom: f32,
    offset: Vec2,
}

impl Default for ViewerState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            offset: Vec2::default(),
        }
    }
}

impl<'a> NesImageViewer<'a> {
    pub(crate) fn new(
        id: &str,
        image: &'a RetainedImage,
        current_pallet: u8,
        pallet_data: &'a mut ImagePalletData,
    ) -> Self {
        let id = egui::Id::new(id);

        Self {
            id,
            image,
            pallet: pallet_data,
            current_pallet,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let gl = frame.gl().unwrap();

        let (rect, mut response) =
            ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());

        // Pan the image
        let drag_delta =
            if response.dragged_by(egui::PointerButton::Middle) || ui.input().modifiers.command {
                response.drag_delta()
            } else {
                Vec2::ZERO
            };
        if response.dragged_by(egui::PointerButton::Middle)
            || (ui.input().modifiers.command && response.dragged_by(egui::PointerButton::Primary))
        {
            response = response.on_hover_cursor(egui::CursorIcon::Grabbing);
        } else if ui.input().modifiers.command {
            response = response.on_hover_cursor(egui::CursorIcon::Grab);
        } else {
            response = response.on_hover_cursor(egui::CursorIcon::Crosshair);
        }

        // Zoom the image
        let zoom_delta = ui.input().scroll_delta.y * 0.01;

        let (renderer, state) = {
            let mut data_store = ui.data();

            let state = {
                let state = data_store.get_temp_mut_or_default::<ViewerState>(self.id);

                state.offset += drag_delta;
                state.zoom += zoom_delta;
                state.zoom = state.zoom.max(0.1);
                *state
            };

            let renderer = data_store
                .get_temp_mut_or_insert_with(self.id, || Arc::new(Mutex::new(Renderer::new(gl))))
                .clone();

            (renderer, state)
        };

        let tile_pallets = self.pallet.tile_pallets.iter().map(|&x| x as u32).collect();

        let paint_info = PaintInfo {
            texture_id: self.image.texture_id(ui.ctx()),
            viewport_size: rect.size(),
            image_size: self.image.size_vec2(),
            offset: state.offset,
            zoom: state.zoom,
            pallet: self.pallet.get_full_pallet_16(),
            tile_pallets,
        };

        let painter = ui.painter_at(rect);

        // Paint the image
        let image_painter = egui::PaintCallback {
            rect,
            callback: std::sync::Arc::new(move |_info, render_ctx| {
                if let Some(painter) = render_ctx.downcast_ref::<egui_glow::Painter>() {
                    renderer.lock().paint(painter, &paint_info);
                } else {
                    eprintln!("Can't do custom painting because we are not using a glow context");
                }
            }),
        };
        painter.add(image_painter);

        // Paint the mouse hover rectangle
        if let Some(mouse_pos) = response.hover_pos() {
            let center = rect.min.to_vec2() + rect.size() / 2.0;
            let image_min = center - self.image.size_vec2() / 2.0 * state.zoom + state.offset;
            let image_max = image_min + self.image.size_vec2() * state.zoom;

            let image_rect = Rect {
                min: image_min.to_pos2(),
                max: image_max.to_pos2(),
            };

            if image_rect.contains(mouse_pos) {
                let scaled_tile_size = TILE_SIZE * state.zoom;
                let mouse_image_pos =
                    (mouse_pos - image_rect.min) / image_rect.size() * self.image.size_vec2();
                let highlight_rect_image_pos = (mouse_image_pos / TILE_SIZE).floor();
                let min = image_rect.min + highlight_rect_image_pos * scaled_tile_size;
                let max = min + scaled_tile_size;
                let tile_rect = Rect { min, max };

                painter.rect_stroke(
                    tile_rect,
                    state.zoom * 0.5,
                    (state.zoom * 0.5, Color32::GREEN),
                );

                let image_tiles_wide = self.image.size()[0] / TILE_SIZE_INT[0];
                let mouse_tile_idx = highlight_rect_image_pos.y as usize * image_tiles_wide
                    + highlight_rect_image_pos.x as usize;

                if response.is_pointer_button_down_on() && ui.ctx().input().pointer.primary_down() {
                    response.mark_changed();
                    self.pallet.tile_pallets[mouse_tile_idx] = self.current_pallet;
                }
            }
        }
    }
}

struct Renderer {
    program: glow::Program,
    va: glow::VertexArray,
    vb: glow::Buffer,
}

struct PaintInfo {
    texture_id: egui::TextureId,
    viewport_size: Vec2,
    image_size: Vec2,
    offset: Vec2,
    zoom: f32,
    pallet: [u32; 16],
    tile_pallets: Vec<u32>,
}

impl Renderer {
    fn new(gl: &glow::Context) -> Self {
        let shader_version = if cfg!(target_arch = "wasm32") {
            "#version 300 es"
        } else {
            "#version 410"
        };

        unsafe {
            let program = gl.create_program().expect("Cannot create program");

            let (vertex_shader_source, fragment_shader_source) = (
                r#"
                const vec2 verts[6] = vec2[6](
                    vec2(0.0, 0.0),
                    vec2(1.0, 0.0),
                    vec2(1.0, 1.0),
                    vec2(1.0, 1.0),
                    vec2(0.0, 1.0),
                    vec2(0.0, 0.0)
                );
                const vec2 uvs[6] = vec2[6](
                    vec2(0.0, 0.0),
                    vec2(1.0, 0.0),
                    vec2(1.0, 1.0),
                    vec2(1.0, 1.0),
                    vec2(0.0, 1.0),
                    vec2(0.0, 0.0)
                );
                const float TILE_SIZE = 16;
                
                uniform vec2 u_viewport_size;
                uniform vec2 u_image_size;
                uniform vec2 u_offset;
                uniform float u_zoom;


                layout (location = 0) in uint in_pallet_idx;
                flat out uint v_pallet_idx;
                out vec2 v_uv;

                void main() {
                    // The size of a pixel in viewport space
                    vec2 v_size_pixel = 2 / u_viewport_size;

                    // The size of a tile in viewport space
                    vec2 v_size_tile = v_size_pixel * TILE_SIZE;

                    // The size of a pixel in UV space
                    vec2 u_size_pixel = 1.0 / u_image_size;

                    // The size of a tile in UV space
                    vec2 u_size_tile = u_size_pixel * TILE_SIZE;

                    uint tile_idx = gl_VertexID / 6;
                    uint image_tiles_wide = uint(u_image_size.x) / uint(TILE_SIZE);
                    uint tile_x = tile_idx % image_tiles_wide;
                    uint tile_y = tile_idx / image_tiles_wide;
                    vec2 tile_xy = vec2(float(tile_x), float(tile_y));

                    vec2 tile_corner = tile_xy * v_size_tile;
                    vec2 pos = tile_xy * v_size_tile + v_size_tile * verts[gl_VertexID % 6];

                    v_uv = uvs[gl_VertexID % 6] * u_size_tile + tile_xy * u_size_tile;
                    v_pallet_idx = in_pallet_idx;

                    vec2 pos_centered = pos - u_image_size * v_size_pixel / 2.0;
                    vec2 pos_zoomed = pos_centered * u_zoom;
                    gl_Position = vec4(pos_zoomed + u_offset * v_size_pixel, 0, 1);
                }
            "#,
                r#"
                precision mediump float;

                in vec2 v_uv;
                flat in uint v_pallet_idx;
                out vec4 out_color;

                uniform uint[16] u_pallet;
                uniform sampler2D u_texture;

                #NES_PALLET

                void main() {
                    vec4 pixel = texture(u_texture, v_uv);
                    // Enumerate the pixel value as one of the four pallet colors
                    uint idx = uint(ceil(pixel.x * 3));

                    out_color = vec4(NES_PALLET[u_pallet[idx + v_pallet_idx * 4]], 1);
                    // out_color = vec4(vec3(float(v_pallet_idx) / 3), 1);
                    // out_color = vec4(v_uv, 0, 1);
                }
            "#
                .replace("#NES_PALLET", &*NES_PALLET_SHADER_CONST),
            );

            let shader_sources = [
                (glow::VERTEX_SHADER, vertex_shader_source),
                (glow::FRAGMENT_SHADER, &fragment_shader_source),
            ];

            let shaders: Vec<_> = shader_sources
                .iter()
                .zip(["Vertex", "Fragment"])
                .map(|((shader_type, shader_source), shader_type_name)| {
                    let shader = gl
                        .create_shader(*shader_type)
                        .expect("Cannot create shader");
                    gl.shader_source(shader, &format!("{}\n{}", shader_version, shader_source));
                    gl.compile_shader(shader);

                    if !gl.get_shader_compile_status(shader) {
                        panic!(
                            "{} shader compile error:\n\n{}\n\nShader Source:\n{}",
                            shader_type_name,
                            gl.get_shader_info_log(shader),
                            shader_source
                                .split('\n')
                                .enumerate()
                                .map(|(i, l)| format!("{}: {}", i, l))
                                .collect::<Vec<_>>()
                                .join("\n")
                        );
                    }

                    gl.attach_shader(program, shader);
                    shader
                })
                .collect();

            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                panic!("{}", gl.get_program_info_log(program));
            }

            for shader in shaders {
                gl.detach_shader(program, shader);
                gl.delete_shader(shader);
            }

            let va = gl.create_vertex_array().expect("Create vertex array");
            let vb = gl.create_buffer().expect("Create vertext buffer");

            Self { program, va, vb }
        }
    }

    fn paint(&mut self, painter: &egui_glow::Painter, info: &PaintInfo) {
        let gl = painter.gl();
        let texture = painter
            .get_texture(info.texture_id)
            .expect("Missing texture");

        unsafe {
            gl.use_program(Some(self.program));
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));

            gl.uniform_1_i32(
                gl.get_uniform_location(self.program, "u_texture").as_ref(),
                0,
            );
            gl.uniform_2_f32(
                gl.get_uniform_location(self.program, "u_offset").as_ref(),
                info.offset.x,
                info.offset.y,
            );
            gl.uniform_1_f32(
                gl.get_uniform_location(self.program, "u_zoom").as_ref(),
                info.zoom,
            );
            gl.uniform_2_f32(
                gl.get_uniform_location(self.program, "u_image_size")
                    .as_ref(),
                info.image_size.x,
                info.image_size.y,
            );
            gl.uniform_1_u32_slice(
                gl.get_uniform_location(self.program, "u_pallet").as_ref(),
                &info.pallet[..],
            );
            gl.uniform_2_f32(
                gl.get_uniform_location(self.program, "u_viewport_size")
                    .as_ref(),
                info.viewport_size.x,
                -info.viewport_size.y,
            );

            let image_size_in_tiles = info.image_size / TILE_SIZE;
            let tile_count = (image_size_in_tiles.x * image_size_in_tiles.y).floor() as i32;
            assert_eq!(
                tile_count as usize,
                info.tile_pallets.len(),
                "There must be one tile pallet entry per tile"
            );

            let data = info
                .tile_pallets
                .iter()
                // For the six vertices that make up each tile, they all have the same pallet
                .flat_map(|&x| [x; 6])
                .collect::<Vec<_>>();

            gl.bind_vertex_array(Some(self.va));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vb));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&data),
                glow::DYNAMIC_DRAW,
            );
            gl.vertex_attrib_pointer_i32(
                0,
                1,
                glow::UNSIGNED_INT,
                std::mem::size_of::<u32>() as i32,
                0,
            );
            gl.enable_vertex_attrib_array(0);

            gl.draw_arrays(glow::TRIANGLES, 0, 6 * tile_count);
        }
    }
}
