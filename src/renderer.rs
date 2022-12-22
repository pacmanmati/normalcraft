use std::borrow::Borrow;

use bytemuck::{Pod, Zeroable};
use fxhash::FxHashMap;
use image::{DynamicImage, GenericImage, GenericImageView, RgbaImage};
use wgpu::{
    util::DeviceExt, vertex_attr_array, Adapter, DepthBiasState, DepthStencilState, StencilState,
    Surface, SurfaceConfiguration,
};
use winit::window::Window;

use crate::{
    camera::Camera,
    instance::{self},
    texture::{self, Texture, TextureAtlas, TextureHandle},
    world::World,
};

pub fn v(x: f32, y: f32, z: f32, u: f32, v: f32) -> Vertex {
    Vertex {
        positions: [x, y, z],
        tex: [u, v],
    }
}

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy)]
pub struct Vertex {
    positions: [f32; 3],
    tex: [f32; 2],
}

struct Object {
    vertex_data: Vec<u8>,
    index_data: Vec<u8>,
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    indices_length: usize,
}

impl std::hash::Hash for Object {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.vertex_data.hash(state);
        self.index_data.hash(state);
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        self.vertex_data == other.vertex_data && self.index_data == other.index_data
    }
}
impl Eq for Object {}

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy)]
struct RenderInstance {
    raw: [f32; 16],
    tex_offset: [f32; 2],
    tex_size: [f32; 2],
}

#[allow(dead_code)]
pub struct RendererBase {
    instance: wgpu::Instance,
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

#[allow(dead_code)]
pub struct Renderer {
    base: RendererBase,
    pipeline: wgpu::RenderPipeline,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    vertices_length: u32,
    indices_length: u32,
    camera_bg: wgpu::BindGroup,
    camera_buffer: wgpu::Buffer,
    depth_texture: Texture,
    objects: FxHashMap<Object, Vec<RenderInstance>>,
    texture_atlas: TextureAtlas,
    textures: FxHashMap<TextureHandle, DynamicImage>,
    texture_atlas_tex: wgpu::Texture,
    sampler: wgpu::Sampler,
    texture_atlas_bg: wgpu::BindGroup,
    texture_atlas_extend: wgpu::Extent3d,
    texture_atlas_bgl: wgpu::BindGroupLayout,
}

impl Renderer {
    pub fn new(window: &winit::window::Window, camera: &Camera) -> Self {
        let base = Self::init(window);

        let module = base
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            });

        let camera_buffer = base
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera buffer"),
                contents: bytemuck::cast_slice(&camera.compute().to_cols_array()),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let camera_bgl = base
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let camera_bg = base.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera bind group"),
            layout: &camera_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &camera_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        // let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = base.device.create_sampler(&wgpu::SamplerDescriptor {
            // address_mode_u: wgpu::AddressMode::Repeat,
            // address_mode_v: wgpu::AddressMode::Repeat,
            // border_color: Some(wgpu::SamplerBorderColor::),
            ..Default::default()
        });

        let texture_bgl = base
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
        // let texture_bg = base.device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     label: Some("Texture bind group"),
        //     layout: &texture_bgl,
        //     entries: &[
        //         wgpu::BindGroupEntry {
        //             binding: 0,
        //             resource: wgpu::BindingResource::TextureView(&texture_view),
        //         },
        //         wgpu::BindGroupEntry {
        //             binding: 1,
        //             resource: wgpu::BindingResource::Sampler(&sampler),
        //         },
        //     ],
        // });

        // let texture_atlas_buffer = base.device.create_buffer(&wgpu::BufferDescriptor {
        //     label: Some("Texture atlas buffer"),
        //     size: 0,
        //     usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        //     mapped_at_creation: false,
        // });

        let texture_size = wgpu::Extent3d {
            width: 512,
            height: 512,
            depth_or_array_layers: 1,
        };

        let texture_atlas_tex = base.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture atlas texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        let texture_view = texture_atlas_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let texture_atlas_bg = base.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture bind group"),
            layout: &texture_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let pipeline_layout = base
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&camera_bgl, &texture_bgl],
                push_constant_ranges: &[],
            });
        let pipeline = base
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: "vertex",
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<Vertex>() as u64,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &vertex_attr_array![0 => Float32x3, 1 => Float32x2],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<RenderInstance>() as u64,
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &vertex_attr_array![2 => Float32x4, 3 => Float32x4, 4 => Float32x4, 5 => Float32x4, 6 => Float32x2, 7 => Float32x2],
                        },
                    ],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    ..Default::default()
                },
                depth_stencil: Some(DepthStencilState{ format: texture::Texture::DEPTH_FORMAT, depth_write_enabled: true, depth_compare: wgpu::CompareFunction::Less, stencil: StencilState::default(), bias: DepthBiasState::default() }),
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: "fragment",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: base.surface.get_supported_formats(&base.adapter)[0],
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::all(),
                    })],
                }),
                multiview: None,
            });

        let vertices_data = crate::world::cube_vertices();

        let vertices = base
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex buffer"),
                contents: bytemuck::cast_slice(&vertices_data),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let indices_data = crate::world::cube_indices();

        let indices = base
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index buffer"),
                contents: bytemuck::cast_slice(&indices_data),
                usage: wgpu::BufferUsages::INDEX,
            });

        let depth_texture = texture::Texture::create_depth_texture(
            &base.device,
            &Self::get_surface_config(&base.adapter, window, &base.surface),
        );

        Self {
            base,
            pipeline,
            camera_bg,
            vertices,
            indices,
            vertices_length: vertices_data.len() as u32,
            indices_length: indices_data.len() as u32,
            camera_buffer,
            depth_texture,
            objects: FxHashMap::default(),
            texture_atlas: TextureAtlas::new(),
            textures: FxHashMap::default(),
            texture_atlas_tex,
            sampler,
            texture_atlas_bg,
            texture_atlas_extend: texture_size,
            texture_atlas_bgl: texture_bgl,
        }
    }

    pub fn init(window: &winit::window::Window) -> RendererBase {
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let (adapter, device, queue) = pollster::block_on(async {
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptionsBase::default())
                .await
                .unwrap();
            let (device, queue) = adapter
                .request_device(&wgpu::DeviceDescriptor::default(), None)
                .await
                .unwrap();
            (adapter, device, queue)
        });

        let surface_config = Self::get_surface_config(&adapter, window, &surface);

        surface.configure(&device, &surface_config);

        RendererBase {
            instance,
            surface,
            adapter,
            device,
            queue,
        }
    }

    fn get_surface_config(
        adapter: &Adapter,
        window: &Window,
        surface: &Surface,
    ) -> SurfaceConfiguration {
        wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(adapter)[0],
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface.get_supported_alpha_modes(adapter)[0],
        }
    }

    pub fn register_texture(&mut self, texture: DynamicImage) -> TextureHandle {
        let rect = texture.borrow().into();
        let handle = self.texture_atlas.add(rect);
        self.textures.insert(handle, texture);
        self.texture_atlas.pack();
        self.update_texture_buffer();
        handle
    }

    fn update_texture_buffer(&mut self) {
        // create texture from atlas and textures
        // how do we go from atlas to texture?
        // make a tex
        // iterate over handles, get from atlas and place at rect location
        // let pixel_size = std::mem::size_of::<[u8; 4]>();

        let mut mega_texture = DynamicImage::ImageRgba8(RgbaImage::new(
            self.texture_atlas.width,
            self.texture_atlas.height,
        ));
        self.textures.iter().for_each(|(handle, image)| {
            let (rect, _) = self.texture_atlas.get_rect(handle).unwrap();
            for (x, y, pixel) in image.pixels() {
                mega_texture.put_pixel(x + rect.x, y + rect.y, pixel)
            }
        });
        // self.texture_atlas;
        let binding = mega_texture.to_rgba8();
        let data = bytemuck::cast_slice(&binding);
        let size = data.len();
        // self.texture_atlas_tex
        //     .create_view(&wgpu::TextureViewDescriptor::default());
        let tex_size = self.texture_atlas_extend.width * self.texture_atlas_extend.height;
        if tex_size >= size.try_into().unwrap() {
            // create a bigger buffer and write to it
            let texture_size = wgpu::Extent3d {
                width: self.texture_atlas.width,
                height: self.texture_atlas.height,
                depth_or_array_layers: 1,
            };
            self.texture_atlas_extend = texture_size;
            self.texture_atlas_tex = self.base.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Texture atlas texture"),
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            });
            // self.base
            //     .device
            //     .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            //         label: Some("Texture atlas buffer"),
            //         contents: data,
            //         usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            //     });
        } else {
            // update the buffer
            println!("tex_size: {tex_size}, ");
            self.base.queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &self.texture_atlas_tex,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                data,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: std::num::NonZeroU32::new(4 * mega_texture.dimensions().0),
                    rows_per_image: std::num::NonZeroU32::new(mega_texture.dimensions().1),
                },
                self.texture_atlas_extend,
            );
        }

        // recreate the view
        let texture_view = self
            .texture_atlas_tex
            .create_view(&wgpu::TextureViewDescriptor::default());

        // recreate the bg
        self.texture_atlas_bg = self
            .base
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Texture bind group"),
                layout: &self.texture_atlas_bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });
    }

    fn create_object(&self, v: Vec<u8>, i: Vec<u8>, indices_length: usize) -> Object {
        Object {
            vertex_data: v,
            index_data: i,
            vertex_buffer: None,
            index_buffer: None,
            indices_length,
        }
    }

    fn register_object(&mut self, mut object: Object, instance: Option<RenderInstance>) {
        let vertices = self
            .base
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex buffer"),
                contents: bytemuck::cast_slice(&object.vertex_data),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let indices = self
            .base
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index buffer"),
                contents: bytemuck::cast_slice(&object.index_data),
                usage: wgpu::BufferUsages::INDEX,
            });
        object.vertex_buffer = Some(vertices);
        object.index_buffer = Some(indices);
        self.objects.insert(
            object,
            if let Some(instance) = instance {
                vec![instance]
            } else {
                vec![]
            },
        );
    }

    pub fn queue_draw(&mut self, drawable: &dyn Drawable, world: &World) {
        // println!("{}", self.objects.keys().len());

        // compare vertex and index data against what we already have to allow efficient drawing
        // if not existing, register it under a new bucket
        let v_data: Vec<u8> = bytemuck::cast_slice(&drawable.vertices()).to_vec();
        let i_data: Vec<u8> = bytemuck::cast_slice(&drawable.indices()).to_vec();
        let object = self.create_object(v_data, i_data, drawable.indices().len());
        let instance = drawable.instance(world);
        let rect = self
            .texture_atlas
            .get_rect(&instance.texture)
            .unwrap_or_else(|| panic!("No rect found for texture with handle {}", instance.texture))
            .0;
        let render_instance = RenderInstance {
            raw: instance.raw(),
            tex_offset: [rect.x as f32, rect.y as f32],
            tex_size: [rect.w as f32, rect.h as f32],
        };
        if !self.objects.contains_key(&object) {
            // register this object
            self.register_object(object, Some(render_instance));
        } else {
            let v = self.objects.get_mut(&object);
            if let Some(instances) = v {
                instances.push(render_instance);
            } else {
                panic!("Expected to find Object in Renderer.")
            }
        }
    }

    pub fn draw(&mut self) {
        let instance_buffer = self.base.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance buffer"),
            size: std::mem::size_of::<RenderInstance>() as u64 * 32 * 32 * 32, // bytes - what's a reasonable limit?
            // what's the most instances of something we might need to draw?
            // keep in mind - all blocks will potentially be of the same instance (e.g. 1 draw call)
            // suppose a render distance of 100 blocks in each direction
            // 100 - you - 100
            // 200 * 200 * 200 * 64 = 512_000_000 bytes which is a 512mb on the gpu.
            // we could probably go 2x that and still work on most hardware?
            // we could also compute the size of buffer required quite easily and implement logic to split it up if necessary
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let frame = self.base.surface.get_current_texture().unwrap();

        let view = &frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .base
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.1,
                        b: 0.5,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        // draw commands
        rpass.set_pipeline(&self.pipeline);
        // rpass.set_vertex_buffer(0, self.vertices.slice(..));
        // rpass.set_vertex_buffer(1, self.instances.slice(..));
        rpass.set_index_buffer(self.indices.slice(..), wgpu::IndexFormat::Uint16);
        rpass.set_bind_group(0, &self.camera_bg, &[]);
        rpass.set_bind_group(1, &self.texture_atlas_bg, &[]);
        // rpass.draw(0..self.vertices_length, 0..1);
        // rpass.draw_indexed(0..self.indices_length, 0, 0..self.instances_length);

        for (object, instances) in self.objects.iter_mut() {
            rpass.set_vertex_buffer(0, object.vertex_buffer.as_ref().unwrap().slice(..));
            rpass.set_index_buffer(
                object.index_buffer.as_ref().unwrap().slice(..),
                wgpu::IndexFormat::Uint16,
            );

            // instance_buffer =
            //     self.base
            //         .device
            //         .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            //             label: Some("Instance buffer"),
            //             contents: bytemuck::cast_slice(&instances),
            //             usage: wgpu::BufferUsages::VERTEX,
            //         });

            self.base
                .queue
                .write_buffer(&instance_buffer, 0, bytemuck::cast_slice(instances));

            rpass.set_vertex_buffer(1, instance_buffer.slice(..));

            rpass.draw_indexed(
                0..object.indices_length as u32,
                0,
                0..instances.len() as u32,
            );
            instances.clear();
        }

        drop(rpass);

        self.base.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    pub fn update_camera(&mut self, camera: &Camera) {
        self.base.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&camera.compute().to_cols_array()),
        );
    }
}

pub trait Drawable {
    fn draw(&self, renderer: &mut Renderer, world: &World);
    fn vertices(&self) -> Vec<Vertex>;
    fn indices(&self) -> Vec<u16>;
    fn instance(&self, world: &World) -> instance::Instance;
}

// registering could be implicit, i.e. through hashing against vertex and index data?
// what we actually want is something like:
// add/register object
// queue instance of the object
// different blocks would become separate instance draws unless we use
// texture arrays and include material data inside of instance data (it makes sense to do so)
