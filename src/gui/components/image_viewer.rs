use std::{borrow::Cow, sync::Arc};

use eframe::{
    egui,
    egui_wgpu::{renderer::CallbackFn, winit::RenderState},
    wgpu,
};
use egui::{mutex::Mutex, Color32, Rect, Vec2};

use egui_extras::RetainedImage;
use glow::HasContext;

use crate::{
    globals::{NES_PALLET_SHADER_CONST, TILE_SIZE, TILE_SIZE_INT},
    gui::ImagePalletData,
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
        let render_state = frame.render_state.clone().expect("WGPU not enabled");

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
                .get_temp_mut_or_insert_with(self.id, || {
                    Arc::new(Mutex::new(Renderer::new(&render_state)))
                })
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
            callback: Arc::new(CallbackFn::new().paint(move |_info, rpass, resources| {
                renderer.lock().paint(
                    rpass,
                    &paint_info,
                    &resources.get().expect("Missing render resources"),
                );
            })),
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

struct Renderer;

struct RenderPassResources {
    pipeline: wgpu::RenderPipeline,
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
    fn new(render_state: &RenderState) -> Self {
        let device = &render_state.device;
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "./image_viewer/shader.wgsl"
            ))),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[render_state.target_format.into()],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        render_state
            .egui_rpass
            .write()
            .paint_callback_resources
            .insert(RenderPassResources { pipeline });

        Self
    }

    fn paint<'rpass>(
        &mut self,
        rpass: &mut wgpu::RenderPass<'rpass>,
        _info: &PaintInfo,
        resources: &'rpass RenderPassResources,
    ) {
        rpass.set_pipeline(&resources.pipeline);
        rpass.draw(0..3, 0..1);
    }
}
