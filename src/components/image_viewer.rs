use std::sync::Arc;

use eframe::egui;
use egui::{mutex::Mutex, Color32, Rect, Vec2};

use egui_extras::RetainedImage;
use glow::HasContext;

use crate::globals::NES_PALETTE_SHADER_CONST;

pub struct NesImageViewer<'a> {
    id: egui::Id,
    image: &'a RetainedImage,
    palette: &'a [u8; 13],
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

const TILE_SIZE_INT: [usize; 2] = [16, 16];
const TILE_SIZE: Vec2 = Vec2::new(TILE_SIZE_INT[0] as f32, TILE_SIZE_INT[1] as f32);

impl<'a> NesImageViewer<'a> {
    pub fn new(id: &str, image: &'a RetainedImage, palette: &'a [u8; 13]) -> Self {
        let id = egui::Id::new(id);

        Self { id, image, palette }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let gl = frame.gl().unwrap();

        let (rect, mut response) = ui.allocate_exact_size(ui.available_size(), egui::Sense::drag());

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
                state.clone()
            };

            let renderer = data_store
                .get_temp_mut_or_insert_with(self.id, || Arc::new(Mutex::new(Renderer::new(gl))))
                .clone();

            (renderer, state)
        };

        let paint_info = PaintInfo {
            texture_id: self.image.texture_id(ui.ctx()),
            viewport_size: rect.size(),
            image_size: self.image.size_vec2(),
            offset: state.offset,
            zoom: state.zoom,
            pallet: [
                self.palette[0] as u32,
                self.palette[1] as u32,
                self.palette[2] as u32,
                self.palette[3] as u32,
                self.palette[0] as u32,
                self.palette[4] as u32,
                self.palette[5] as u32,
                self.palette[6] as u32,
                self.palette[0] as u32,
                self.palette[7] as u32,
                self.palette[8] as u32,
                self.palette[9] as u32,
                self.palette[0] as u32,
                self.palette[10] as u32,
                self.palette[11] as u32,
                self.palette[12] as u32,
            ],
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
            }
        }
    }
}

struct Renderer {
    program: glow::Program,
    va: glow::VertexArray,
}

struct PaintInfo {
    texture_id: egui::TextureId,
    viewport_size: Vec2,
    image_size: Vec2,
    offset: Vec2,
    zoom: f32,
    pallet: [u32; 16],
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
                const vec2 verts[4] = vec2[4](
                    vec2(-1.0, -1.0),
                    vec2(1.0, -1.0),
                    vec2(1.0, 1.0),
                    vec2(-1.0, 1.0)
                );
                const vec2 uvs[4] = vec2[4](
                    vec2(0.0, 0.0),
                    vec2(1.0, 0.0),
                    vec2(1.0, 1.0),
                    vec2(0.0, 1.0)
                );
                
                uniform vec2 u_viewport_size;
                uniform vec2 u_image_size;
                uniform vec2 u_offset;
                uniform float u_zoom;

                out vec2 v_uv;

                void main() {
                    v_uv = uvs[gl_VertexID];
                    gl_Position = vec4(verts[gl_VertexID], 0.0, 1.0) / 2 / vec4(u_viewport_size, 1, 1) * 
                        vec4(u_image_size * u_zoom, 1, 1) + vec4(u_offset / u_viewport_size, 0, 0);
                }
            "#,
                r#"
                precision mediump float;

                in vec2 v_uv;
                out vec4 out_color;

                uniform uint[16] u_pallet;
                uniform sampler2D u_texture;

                #NES_PALETTE

                void main() {
                    vec4 pixel = texture(u_texture, v_uv);
                    // Enumerate the pixel value as one of the four pallet colors
                    uint idx = uint(ceil(pixel.x * 3));

                    out_color = NES_PALLET[u_pallet[idx]];
                }
            "#
                .replace("#NES_PALETTE", &*NES_PALETTE_SHADER_CONST),
            );

            let shader_sources = [
                (glow::VERTEX_SHADER, vertex_shader_source),
                (glow::FRAGMENT_SHADER, &fragment_shader_source),
            ];

            let shaders: Vec<_> = shader_sources
                .iter()
                .map(|(shader_type, shader_source)| {
                    let shader = gl
                        .create_shader(*shader_type)
                        .expect("Cannot create shader");
                    gl.shader_source(shader, &format!("{}\n{}", shader_version, shader_source));
                    gl.compile_shader(shader);

                    if !gl.get_shader_compile_status(shader) {
                        panic!("Shader compile error: {}", gl.get_shader_info_log(shader));
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

            let va = gl
                .create_vertex_array()
                .expect("Cannot create vertex array");

            Self { program, va }
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

            // let block_index = gl
            //     .get_uniform_block_index(self.program, "Image")
            //     .expect("Missing uniform block index");
            // gl.uniform_block_binding(self.program, block_index, 0);
            // gl.bind_buffer_base(
            //     glow::UNIFORM_BUFFER,
            //     0,
            //     Some(info.image_handle.map_or_get_gpu_buffer(gl)),
            // );

            gl.bind_vertex_array(Some(self.va));
            gl.draw_arrays(glow::TRIANGLE_FAN, 0, 4);
        }
    }
}
