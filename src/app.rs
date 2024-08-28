#[allow(unused_imports)]
use log::{debug, error, info};

use std::{iter, time::Instant};

use anyhow::{Context, Result};

use wgpu::util::DeviceExt;

use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

use crate::Args;
#[allow(unused_imports)]
use crate::{
    colors::Rgba,
    raytracer::{render, RenderMode},
    texture::Texture,
};

#[allow(dead_code)]
pub struct GraphicsState<'a> {
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'a>,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    compute_pipeline: wgpu::ComputePipeline,
    vertex_buffer: wgpu::Buffer,
    compute_parameters_buffer: wgpu::Buffer,
    output_texture_bind_group: wgpu::BindGroup,
    compute_bind_group: wgpu::BindGroup,
    window: &'a Window,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
}

const FULL_SCREEN_QUAD: [Vertex; 6] = [
    Vertex {
        position: [-1.0, 1.0, 0.0],
    },
    Vertex {
        position: [-1.0, -1.0, 0.0],
    },
    Vertex {
        position: [1.0, 1.0, 0.0],
    },
    Vertex {
        position: [1.0, 1.0, 0.0],
    },
    Vertex {
        position: [-1.0, -1.0, 0.0],
    },
    Vertex {
        position: [1.0, -1.0, 0.0],
    },
];

#[allow(dead_code)]
impl<'a> GraphicsState<'a> {
    pub async fn new(window: &'a Window, args: &Args) -> Result<Self> {
        //WGPU Instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        //Adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            })
            .await
            .context("Failed to get adapter")?;
        info!("{:?}", adapter.get_info());

        //Device & Queue
        let (device, queue) = adapter.request_device(&Default::default(), None).await?;

        //Surface
        let surface_size = window.inner_size();

        let surface = instance.create_surface(window)?;

        let surface_capabilities = surface.get_capabilities(&adapter);

        let surface_format = surface_capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: surface_size.width,
            height: surface_size.height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        //Shaders
        let texture_shader_module =
            device.create_shader_module(wgpu::include_wgsl!("shaders/texture/shader.wgsl"));
        let compute_shader_module =
            device.create_shader_module(wgpu::include_wgsl!("shaders/compute/shader.wgsl"));

        //Vertex Buffer
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&FULL_SCREEN_QUAD),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: 0,
                shader_location: 0,
            }],
        };
        //Texture Binding
        let mut texture = Texture::new(
            vec![Rgba::<u8>::default(); (surface_size.width * surface_size.height) as usize],
            surface_size.width as usize,
            surface_size.height as usize,
        );

        match args.mode {
            RenderMode::SingleThread => {
                render(&mut texture, args.samples, RenderMode::SingleThread)?
            }
            RenderMode::MultiThread => render(&mut texture, args.samples, RenderMode::MultiThread)?,
            RenderMode::Gpu => (),
        }

        let texture = texture.into_wgpu_texture(
            &device,
            &queue,
            wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::TEXTURE_BINDING,
        );

        let texture_view = texture.create_view(&Default::default());

        //Texture output bind group
        let output_texture_binding_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Image binding group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                }],
            });

        let output_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Image binding group layout"),
            layout: &output_texture_binding_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            }],
        });

        //Render Pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&output_texture_binding_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("RenderPipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &texture_shader_module,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &[vertex_buffer_layout],
            },
            fragment: Some(wgpu::FragmentState {
                module: &texture_shader_module,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });

        //Compute bind group
        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
            });

        let compute_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 12,
            usage: wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: compute_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
            ],
        });

        //Compute Pipeline
        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Compute pipeline layout"),
                bind_group_layouts: &[&compute_bind_group_layout],
                push_constant_ranges: &[],
            });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader_module,
            entry_point: "main",
            compilation_options: Default::default(),
            cache: None,
        });

        Ok(GraphicsState {
            adapter,
            device,
            queue,
            surface,
            config,
            render_pipeline,
            compute_pipeline,
            vertex_buffer,
            compute_parameters_buffer: compute_buffer,
            output_texture_bind_group,
            window,
            compute_bind_group,
        })
    }

    fn window(&self) -> &Window {
        self.window
    }

    fn resize(&mut self, new_size: &PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn render(&self, args: &Args) -> Result<()> {
        let now = Instant::now();

        let output: wgpu::SurfaceTexture = self.surface.get_current_texture()?;

        let view = output.texture.create_view(&Default::default());

        let mut encoder = self.device.create_command_encoder(&Default::default());

        //Compute if in gpu mode
        if args.mode == RenderMode::Gpu {
            let buffer = [self.config.width, self.config.height, args.samples];
            let compute_buffer =
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(&buffer),
                        usage: wgpu::BufferUsages::COPY_SRC,
                    });
            encoder.copy_buffer_to_buffer(
                &compute_buffer,
                0,
                &self.compute_parameters_buffer,
                0,
                12,
            );

            let mut compute_pass = encoder.begin_compute_pass(&Default::default());
            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);
            compute_pass.dispatch_workgroups(self.config.width / 16, self.config.height / 16, 1);
        }

        //Render (set texture to surface)
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Discard,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_bind_group(0, &self.output_texture_bind_group, &[]);
            render_pass.draw(0..6, 0..1)
        }

        self.queue.submit(iter::once(encoder.finish()));

        output.present();

        if args.mode == RenderMode::Gpu {
            let elapsed = now.elapsed();
            info!("Elapsed: {:.2?}", elapsed);
        }

        Ok(())
    }
}

//Hols the app state
pub struct Application<'a> {
    state: GraphicsState<'a>,
    args: Args,
}

impl<'a> Application<'a> {
    async fn new(window: &'a Window, args: Args) -> Result<Self> {
        let state = GraphicsState::new(window, &args).await?;

        Ok(Application { state, args })
    }

    fn render(&self) -> Result<()> {
        self.state.render(&self.args)?;

        Ok(())
    }

    fn update(&self) {}

    fn input(&mut self, event: &WindowEvent) -> bool {
        matches!(
            event,
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                    state: _,
                    ..
                },
                ..
            }
        )
    }

    pub async fn run(args: Args) -> Result<()> {
        let event_loop = EventLoop::new().expect("Failed to create event_loop");
        event_loop.set_control_flow(ControlFlow::Poll);

        let scale = 1.0;

        //Create the window
        let window = WindowBuilder::new()
            .with_title("WGPU_Raytracer")
            .with_resizable(false)
            .with_inner_size(PhysicalSize {
                width: 1920.0 * scale,
                height: 1080.0 * scale,
            })
            .build(&event_loop)?;

        //Create the main application
        let mut app = Application::new(&window, args).await?;

        //Main loop
        event_loop.run(move |event, window| match event {
            Event::WindowEvent {
                ref event,
                window_id: _,
            } => {
                if !app.input(event) {
                    match event {
                        //Close
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    state: ElementState::Pressed,
                                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                                    ..
                                },
                            ..
                        } => window.exit(),
                        //Resize
                        WindowEvent::Resized(physical_size) => {
                            app.state.resize(physical_size);
                        }
                        //Redraw
                        WindowEvent::RedrawRequested => {
                            app.update();

                            if let Err(e) = app.render() {
                                error!("{e}");
                            };
                        }
                        _ => (),
                    }
                }
            }
            Event::AboutToWait => app.state.window.request_redraw(),
            _ => {}
        })?;
        Ok(())
    }
}
