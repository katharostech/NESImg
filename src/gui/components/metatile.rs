use std::{collections::HashMap, sync::Arc};

use eframe::{
    egui,
    egui_wgpu::{renderer::CallbackFn, winit::RenderState},
    wgpu::{self, util::DeviceExt},
};

use crate::{
    constants::NES_PALLET_SHADER_CONST,
    gui::project_state::{ProjectState, SourceImageStatus},
    project::{Metatile, Metatileset, MetatilesetTile},
    Uid,
};

#[derive(Hash)]
pub enum MetatileKind {
    Standalone(Uid<Metatile>),
    Metatileset {
        metatileset_id: Uid<Metatileset>,
        metatileset_tile_id: Uid<MetatilesetTile>,
    },
}

impl MetatileKind {
    fn get_metatile<'a, 'b>(&'a self, project: &'b ProjectState) -> Option<&'b Metatile> {
        match self {
            MetatileKind::Standalone(metatile_id) => project.data.metatiles.get(metatile_id),
            MetatileKind::Metatileset {
                metatileset_id,
                metatileset_tile_id,
            } => project
                .data
                .metatilesets
                .get(metatileset_id)
                .and_then(|metatileset| metatileset.tiles.get(metatileset_tile_id))
                .map(|metatileset_tile| metatileset_tile.metatile_id)
                .and_then(|metatile_id| project.data.metatiles.get(&metatile_id)),
        }
    }
}

pub struct MetatileGui<'a> {
    tile: MetatileKind,
    project: &'a mut ProjectState,
}

impl<'a> MetatileGui<'a> {
    #[must_use = "Must call .show() to display"]
    pub fn new(project: &'a mut ProjectState, tile: MetatileKind) -> Self {
        Self { tile, project }
    }

    // pub fn show(&mut self, size: egui::Vec2, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
    //     let (rect, _response) = ui.allocate_exact_size(size, egui::Sense::hover());
    //     self.show_at(rect, ui, frame);
    // }

    pub fn show_at(&mut self, rect: egui::Rect, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let render_state = frame.render_state.clone().expect("WGPU not enabled");

        Renderer::ensure_init(&render_state);

        let painter = ui.painter_at(rect);

        let mut raw_tiles = Vec::with_capacity(4);

        if self.tile.get_metatile(self.project).is_none() {
            return;
        }

        for i in 0..4 {
            let mut get_tile = || {
                let tile = self.tile.get_metatile(self.project).unwrap().tiles[i].clone()?;
                let source_image = self.project.source_images.get_mut(&tile.source_id).unwrap();

                let source_image_data =
                    if let SourceImageStatus::Found(image) = source_image.data.get() {
                        image
                    } else {
                        return None;
                    };
                let source_image_size = source_image_data.texture.size_vec2();
                let texture_id = source_image_data.texture.texture_id(ui.ctx());

                let texture_view = render_state
                    .egui_rpass
                    .read()
                    .get_texture(&texture_id)
                    .as_ref()?
                    .0
                    .as_ref()?
                    .create_view(&wgpu::TextureViewDescriptor::default());

                Some(RawTile {
                    texture_view,
                    uv_start: [
                        tile.x as f32 * 8.0 / source_image_size.x,
                        tile.y as f32 * 8.0 / source_image_size.y,
                    ],
                    uv_size: [8.0 / source_image_size.x, 8.0 / source_image_size.y],
                })
            };
            raw_tiles.push(get_tile());
        }
        let raw_tiles = [
            raw_tiles.remove(0),
            raw_tiles.remove(0),
            raw_tiles.remove(0),
            raw_tiles.remove(0),
        ];

        let colors = match &self.tile {
            MetatileKind::Standalone { .. } => [0x0F, 0x2D, 0x00, 0x30],
            MetatileKind::Metatileset {
                metatileset_id,
                metatileset_tile_id,
            } => {
                let metatileset = self.project.data.metatilesets.get(metatileset_id).unwrap();

                let sub_pallet_idx = metatileset
                    .tiles
                    .get(metatileset_tile_id)
                    .map(|metatileset_tile| metatileset_tile.sub_pallet_idx)
                    .unwrap();
                metatileset.pallet.get_sub_pallets()[sub_pallet_idx]
            }
        };

        let id = ui.id().with(&self.tile);

        // Paint the image
        let image_painter = egui::PaintCallback {
            rect,
            callback: Arc::new(
                CallbackFn::new()
                    .prepare(move |device, queue, resources| {
                        let renderer: &mut Renderer = resources.get_mut().unwrap();

                        renderer.prepare(device, queue, id, &raw_tiles, colors);
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

struct RawTile {
    texture_view: wgpu::TextureView,
    uv_start: [f32; 2],
    uv_size: [f32; 2],
}

struct MetatileResources {
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
}

struct Renderer {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    metatile_resources: HashMap<egui::Id, MetatileResources>,
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
                    source: wgpu::ShaderSource::Wgsl(
                        include_str!("./metatile/shader.wgsl")
                            .replace("// #NES_PALLET", &NES_PALLET_SHADER_CONST)
                            .into(),
                    ),
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
                    metatile_resources: Default::default(),
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
        queue: &wgpu::Queue,
        id: egui::Id,
        raw_tiles: &[Option<RawTile>; 4],
        colors: [u32; 4],
    ) {
        #[derive(encase::ShaderType)]
        struct MetatileUniform {
            tiles: [MetatileTileUniform; 4],
            colors: glam::UVec4,
        }

        #[derive(encase::ShaderType)]
        struct MetatileTileUniform {
            #[align(16)]
            tex_idx: u32,
            uv_start: glam::Vec2,
            uv_size: glam::Vec2,
        }

        let mut uniform_tiles = raw_tiles
            .iter()
            .enumerate()
            .map(|(i, tile)| {
                if let Some(tile) = tile {
                    MetatileTileUniform {
                        tex_idx: i as u32 + 1,
                        uv_start: tile.uv_start.into(),
                        uv_size: tile.uv_size.into(),
                    }
                } else {
                    MetatileTileUniform {
                        tex_idx: 0,
                        uv_start: [0.0; 2].into(),
                        uv_size: [0.0; 2].into(),
                    }
                }
            })
            .collect::<Vec<_>>();
        let mut uniform_buffer_temp = encase::UniformBuffer::new(Vec::new());
        uniform_buffer_temp
            .write(&MetatileUniform {
                tiles: [
                    uniform_tiles.remove(0),
                    uniform_tiles.remove(0),
                    uniform_tiles.remove(0),
                    uniform_tiles.remove(0),
                ],
                colors: colors.into(),
            })
            .expect("Format uniform buffer");
        let uniform_buffer_bytes = uniform_buffer_temp.into_inner();

        let uniform_buffer = if let Some(resources) = self.metatile_resources.remove(&id) {
            queue.write_buffer(&resources.uniform_buffer, 0, &uniform_buffer_bytes);

            resources.uniform_buffer
        } else {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("metatile"),
                contents: &uniform_buffer_bytes,
                usage: wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::MAP_WRITE
                    | wgpu::BufferUsages::UNIFORM,
            })
        };

        let mut entries = vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1u32,
                resource: wgpu::BindingResource::TextureView(&self.empty_tile_texture_view),
            },
        ];

        for (i, texture) in raw_tiles
            .iter()
            .map(|x| x.as_ref().map(|x| &x.texture_view))
            .enumerate()
        {
            if let Some(tex) = texture {
                entries.push(wgpu::BindGroupEntry {
                    binding: (i + 2) as u32,
                    resource: wgpu::BindingResource::TextureView(tex),
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

        self.metatile_resources.insert(
            id,
            MetatileResources {
                bind_group,
                uniform_buffer,
            },
        );
    }

    fn paint<'rpass>(&'rpass self, rpass: &mut wgpu::RenderPass<'rpass>, id: egui::Id) {
        let MetatileResources { bind_group, .. } = self.metatile_resources.get(&id).unwrap();
        rpass.set_bind_group(0, bind_group, &[]);
        rpass.set_pipeline(&self.pipeline);
        rpass.draw(0..(2 * 3 * 4), 0..1);
    }
}
