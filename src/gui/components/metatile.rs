use std::{borrow::Cow, collections::HashMap, sync::Arc};

use eframe::{
    egui,
    egui_wgpu::{renderer::CallbackFn, winit::RenderState},
    wgpu::{self, util::DeviceExt},
};
use egui::Vec2;

use ulid::Ulid;

use crate::gui::project_state::{ProjectState, SourceImageStatus};

pub struct MetatileGui<'a> {
    id: Ulid,
    project: &'a mut ProjectState,
    size: Vec2,
}

impl<'a> MetatileGui<'a> {
    #[must_use = "Must call .show() to display"]
    pub fn new(project: &'a mut ProjectState, id: Ulid, size: Vec2) -> Self {
        Self { id, project, size }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let (rect, _response) = ui.allocate_exact_size(self.size, egui::Sense::hover());
        self.show_at(rect, ui, frame);
    }

    pub fn show_at(&mut self, rect: egui::Rect, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let render_state = frame.render_state.clone().expect("WGPU not enabled");

        // ui.painter().rect_filled(rect, 0.0, egui::Color32::GOLD);

        Renderer::ensure_init(&render_state);

        let painter = ui.painter_at(rect);

        let mut textures = Vec::with_capacity(4);
        let metatile = if let Some(m) = self.project.data.metatiles.get(&self.id) {
            m
        } else {
            return;
        };

        for i in 0..4 {
            let mut get_texture = || {
                let tile = metatile.tiles[i].as_ref()?;
                let source_image = self.project.source_images.get_mut(&tile.source_id).unwrap();

                let texture_id = if let SourceImageStatus::Found(image) = &source_image.data.get() {
                    image.texture.texture_id(ui.ctx())
                } else {
                    return None;
                };

                let texture = render_state
                    .egui_rpass
                    .read()
                    .textures
                    .get(&texture_id)
                    .as_ref()?
                    .0
                    .as_ref()?
                    .create_view(&wgpu::TextureViewDescriptor::default());

                Some(texture)
            };
            textures.push(get_texture());
        }
        let texture_views = [
            textures.remove(0),
            textures.remove(0),
            textures.remove(0),
            textures.remove(0),
        ];
        let id = self.id;

        // Paint the image
        let image_painter = egui::PaintCallback {
            rect,
            callback: Arc::new(
                CallbackFn::new()
                    .prepare(move |device, queue, resources| {
                        let renderer: &mut Renderer = resources.get_mut().unwrap();

                        renderer.prepare(device, queue, id, &texture_views);
                    })
                    .paint(move |_info, rpass, resources| {
                        let renderer: &Renderer = resources.get().unwrap();

                        renderer.paint(rpass, id);
                    }),
            ),
        };
        painter.add(image_painter);
    }
}

struct Renderer {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_groups: HashMap<Ulid, wgpu::BindGroup>,
    sampler: wgpu::Sampler,
    empty_tile_texture_view: wgpu::TextureView,
}

impl Renderer {
    fn ensure_init(render_state: &RenderState) {
        let device = &render_state.device;
        let queue = &render_state.queue;
        let label = Some("metatile");

        render_state
            .egui_rpass
            .write()
            .paint_callback_resources
            .entry::<Renderer>()
            .or_insert_with(|| {
                let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                    label,
                    source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                        "./metatile/shader.wgsl"
                    ))),
                });

                let bind_group_layout =
                    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        label,
                        entries: &[
                            wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                                ty: wgpu::BindingType::Buffer {
                                    ty: wgpu::BufferBindingType::Uniform,
                                    has_dynamic_offset: false,
                                    min_binding_size: None,
                                },
                                count: None,
                            },
                            wgpu::BindGroupLayoutEntry {
                                binding: 1,
                                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                                ty: wgpu::BindingType::Texture {
                                    sample_type: wgpu::TextureSampleType::Float {
                                        filterable: false,
                                    },
                                    view_dimension: wgpu::TextureViewDimension::D2,
                                    multisampled: false,
                                },
                                count: None,
                            },
                            wgpu::BindGroupLayoutEntry {
                                binding: 2,
                                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                                ty: wgpu::BindingType::Texture {
                                    sample_type: wgpu::TextureSampleType::Float {
                                        filterable: false,
                                    },
                                    view_dimension: wgpu::TextureViewDimension::D2,
                                    multisampled: false,
                                },
                                count: None,
                            },
                            wgpu::BindGroupLayoutEntry {
                                binding: 3,
                                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                                ty: wgpu::BindingType::Texture {
                                    sample_type: wgpu::TextureSampleType::Float {
                                        filterable: false,
                                    },
                                    view_dimension: wgpu::TextureViewDimension::D2,
                                    multisampled: false,
                                },
                                count: None,
                            },
                            wgpu::BindGroupLayoutEntry {
                                binding: 4,
                                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                                ty: wgpu::BindingType::Texture {
                                    sample_type: wgpu::TextureSampleType::Float {
                                        filterable: false,
                                    },
                                    view_dimension: wgpu::TextureViewDimension::D2,
                                    multisampled: false,
                                },
                                count: None,
                            },
                            wgpu::BindGroupLayoutEntry {
                                binding: 5,
                                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                                ty: wgpu::BindingType::Texture {
                                    sample_type: wgpu::TextureSampleType::Float {
                                        filterable: false,
                                    },
                                    view_dimension: wgpu::TextureViewDimension::D2,
                                    multisampled: false,
                                },
                                count: None,
                            },
                            wgpu::BindGroupLayoutEntry {
                                binding: 6,
                                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                                ty: wgpu::BindingType::Sampler(
                                    wgpu::SamplerBindingType::NonFiltering,
                                ),
                                count: None,
                            },
                        ],
                    });

                let pipeline_layout =
                    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label,
                        bind_group_layouts: &[&bind_group_layout],
                        push_constant_ranges: &[],
                    });

                let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label,
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

                let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                    label,
                    ..Default::default()
                });

                let empty_tile_texture = device.create_texture_with_data(
                    queue,
                    &wgpu::TextureDescriptor {
                        label,
                        size: wgpu::Extent3d {
                            width: 1,
                            height: 1,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Rgba32Float,
                        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    },
                    bytemuck::cast_slice(&[0.0, 0.0, 0.0, 1.0]),
                );
                let empty_tile_texture_view =
                    empty_tile_texture.create_view(&wgpu::TextureViewDescriptor::default());

                Renderer {
                    bind_groups: Default::default(),
                    pipeline,
                    bind_group_layout,
                    sampler,
                    empty_tile_texture_view,
                }
            });
    }

    fn prepare(
        &mut self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        id: Ulid,
        textures: &[Option<wgpu::TextureView>; 4],
    ) {
        let tiles = textures
            .iter()
            .enumerate()
            .map(|(x, y)| if y.is_some() { x as u32 + 1 } else { 0 })
            .collect::<Vec<_>>();

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("metatile"),
            contents: bytemuck::cast_slice(&tiles),
            usage: wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::MAP_WRITE
                | wgpu::BufferUsages::UNIFORM,
        });

        let mut entries = vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1 as u32,
                resource: wgpu::BindingResource::TextureView(&self.empty_tile_texture_view),
            },
        ];

        for (i, texture) in textures.iter().enumerate() {
            if let Some(tex) = texture {
                entries.push(wgpu::BindGroupEntry {
                    binding: (i + 2) as u32,
                    resource: wgpu::BindingResource::TextureView(&tex),
                });
            } else {
                entries.push(wgpu::BindGroupEntry {
                    binding: (i + 2) as u32,
                    resource: wgpu::BindingResource::TextureView(&self.empty_tile_texture_view),
                });
            }
        }

        entries.push(wgpu::BindGroupEntry {
            binding: 6,
            resource: wgpu::BindingResource::Sampler(&self.sampler),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("metatile"),
            layout: &self.bind_group_layout,
            entries: &entries,
        });

        self.bind_groups.insert(id, bind_group);
    }

    fn paint<'rpass>(&'rpass self, rpass: &mut wgpu::RenderPass<'rpass>, id: Ulid) {
        let bind_group = self.bind_groups.get(&id).unwrap();
        rpass.set_bind_group(0, bind_group, &[]);
        rpass.set_pipeline(&self.pipeline);
        rpass.draw(0..(2 * 3 * 4), 0..1);
    }
}
