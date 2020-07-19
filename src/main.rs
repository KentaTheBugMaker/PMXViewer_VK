extern crate PMXUtil;
extern crate vulkano as vk;

use std::collections::HashMap;
use std::iter;
use std::sync::Arc;
use std::time::Instant;

use cgmath::{Matrix3, Matrix4, Point3, Rad, Vector3};
use vk::buffer::{BufferUsage, CpuBufferPool};
use vk::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vk::descriptor::descriptor_set::PersistentDescriptorSet;
use vk::descriptor::PipelineLayoutAbstract;
use vk::device::{Device, DeviceExtensions};
use vk::format::Format;
use vk::framebuffer::{FramebufferAbstract, RenderPassAbstract, Subpass};
use vk::framebuffer::Framebuffer;
use vk::image::{AttachmentImage, SwapchainImage, ImageUsage};
use vk::instance::{PhysicalDevice, PhysicalDeviceType};
use vk::pipeline::GraphicsPipeline;
use vk::pipeline::GraphicsPipelineAbstract;
use vk::pipeline::vertex::SingleBufferDefinition;
use vk::pipeline::viewport::Viewport;
use vk::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode};
use vk::swapchain::{AcquireError, ColorSpace, FullscreenExclusive, PresentMode, SurfaceTransform, Swapchain, SwapchainCreationError};
use vk::swapchain;
use vk::sync::{FlushError, GpuFuture};
use vk::sync;
use vulkano::instance::Instance;
use vulkano::instance::layers_list;
use vulkano_win::VkSurfaceBuild;
use winit::event::{DeviceEvent, ElementState, Event, MouseScrollDelta, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

use shaders::fs as fs;
use shaders::vs as vs;

use crate::util::util::{convert_to_vertex_buffer, make_draw_asset, Vertex};

mod shader_transpiler;
mod shader_loader;
mod shaders;
mod util;
mod tokenizer;
fn main() {
    let required_extension = vulkano_win::required_extensions();
    let layer_list = layers_list().unwrap();
    for layer in layer_list {
        println!("Avilable layer:{}", layer.name());
    }
    let instance = Instance::new(None, &required_extension, None).unwrap();
    let physical_device = vk::instance::PhysicalDevice::enumerate(&instance);
    let mut what_gpu_use = HashMap::new();

    //GPUと優先度のマップを作成

    for device in physical_device {
        let name = device.name();
        let device_type = device.ty();
        let mut power = 0;
        //remove emulators
        //dGPU prefer than iGPU
        power += match device_type {
            PhysicalDeviceType::IntegratedGpu => { 1 }
            PhysicalDeviceType::DiscreteGpu => { 2 }
            PhysicalDeviceType::VirtualGpu => { 1 }
            PhysicalDeviceType::Cpu => { -1 }
            PhysicalDeviceType::Other => { -1 }
        };
        what_gpu_use.insert(power, device);
        println!("GPU:{} is {:#?}", name, device_type);
    }
    let mut what_gpu_use: Vec<(i32, &PhysicalDevice)> = what_gpu_use.iter().map(|(k, v)| {
        (*k, v)
    }).collect();
    what_gpu_use.sort_by(|a, b| { a.0.cmp(&b.0) });
    let physical = *what_gpu_use.pop().unwrap().1;

    println!("Chased GPU is {} ", physical.name());
    let event_loop = EventLoop::new();
    let surface = WindowBuilder::new().build_vk_surface(&event_loop, instance.clone()).unwrap();
    let dimensions: [u32; 2] = surface.window().inner_size().into();

    let queue_family = physical.queue_families().find(|&q|
        q.supports_graphics() && surface.is_supported(q).unwrap_or(false)
    ).unwrap();

    let device_ext = DeviceExtensions { khr_swapchain: true, ..DeviceExtensions::none() };

    let (device, mut queues) = Device::new(
        physical, physical.supported_features(), &device_ext, [(queue_family, 0.5)].iter().cloned(),
    ).unwrap();

    let queue = queues.next().unwrap();


    let caps = surface.capabilities(physical).unwrap();

    let formats = caps.supported_formats;
    for fm in formats.iter() {
        println!("Format:{:#?} ColorSpace:{:#?}", fm.0, fm.1);
    }
    let format = formats[0].0;
    let alpha = caps.supported_composite_alpha.iter().next().unwrap();
    let (mut swapchain, images) = {
        Swapchain::new(device.clone(), surface.clone(), caps.min_image_count, format, dimensions, 1,
                       ImageUsage::color_attachment(), &queue, SurfaceTransform::Identity, alpha, PresentMode::Fifo,
                       FullscreenExclusive::Default, true, ColorSpace::SrgbNonLinear).unwrap()
    };
//loading and convert to buffers

    let path = std::env::args().skip(1).next().unwrap();
    let mut loader = PMXUtil::pmx_loader::pmx_loader::PMXLoader::open(&path);
    loader.read_pmx_model_info().unwrap();

    let vertices = loader.read_pmx_vertices().unwrap();
    let mut vertex_buffer = convert_to_vertex_buffer(device.clone(), &vertices);

    let mut faces = loader.read_pmx_faces().unwrap();

    let texture_list = loader.read_texture_list().unwrap();
    let materials = loader.read_pmx_materials().unwrap();

    let (mut assets, mut textures) = make_draw_asset(device.clone(), queue.clone(), &mut faces, &texture_list, &materials, &path);

    let uniform_buffer = CpuBufferPool::<vs::ty::Data>::new(device.clone(), BufferUsage::all());
    let uniform_buffer_screen = CpuBufferPool::<fs::ty::Image>::new(device.clone(), BufferUsage::uniform_buffer());
    let uniform_buffer_material = CpuBufferPool::<fs::ty::Material>::new(device.clone(), BufferUsage::uniform_buffer());
    let vs = vs::Shader::load(device.clone()).unwrap();
    let fs = fs::Shader::load(device.clone()).unwrap();

    let render_pass = Arc::new(
        vulkano::single_pass_renderpass!(device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.format(),
                    samples: 1,
                },
                depth: {
                    load: Clear,
                    store: DontCare,
                    format: Format::D32Sfloat,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {depth}
            }
        ).unwrap()
    );

    let (mut pipeline, mut framebuffers) = window_size_dependent_setup(device.clone(), &vs, &fs, &images, render_pass.clone());
    let mut recreate_swapchain = false;

    let mut previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<dyn GpuFuture>);
    let rotation_start = Instant::now();
    let mut zoom = 0.05;
    let mut translate_x = 0.00;
    let mut translate_y = 0.00;
    let mut flag_move = false;
    let mut zoom_flag = false;
    let mut focus = true;
    let sampler = Sampler::new(device.clone(), Filter::Linear, Filter::Linear,
                               MipmapMode::Nearest, SamplerAddressMode::Repeat, SamplerAddressMode::Repeat,
                               SamplerAddressMode::Repeat, 0.0, 1.0, 0.0, 0.0).unwrap();
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent { event: WindowEvent::Resized(_), .. } => {
                recreate_swapchain = true;
            }
            Event::WindowEvent { event: WindowEvent::Focused(change), .. } => {
                focus = change;
            }
            Event::WindowEvent { event: WindowEvent::DroppedFile(buf), .. } => {
                let path = buf.as_path().to_str().unwrap();
                println!("Path:{}", path);
                let mut loader = PMXUtil::pmx_loader::pmx_loader::PMXLoader::open(&buf);
                loader.read_pmx_model_info().unwrap();

                let vertices = loader.read_pmx_vertices().unwrap();
                vertex_buffer = convert_to_vertex_buffer(device.clone(), &vertices);

                let mut faces = loader.read_pmx_faces().unwrap();

                let texture_list = loader.read_texture_list().unwrap();
                let materials = loader.read_pmx_materials().unwrap();

                textures.clear();
                let da = make_draw_asset(device.clone(), queue.clone(), &mut faces, &texture_list, &materials, path);
                assets = da.0;
                textures = da.1;
            }
            Event::DeviceEvent { event: DeviceEvent::Key(input), .. } => {
                if input.state == ElementState::Pressed {
                    match input.virtual_keycode {
                        Some(code) => {
                            if focus {
                                match code {
                                    VirtualKeyCode::M => {
                                        //move
                                        flag_move = !flag_move;
                                    }
                                    VirtualKeyCode::R => {
                                        //rotate
                                    }
                                    VirtualKeyCode::Z => {
                                        zoom_flag = !zoom_flag;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        None => {}
                    }
                }
            }
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                let (x, y) = delta;
                if focus {
                    if flag_move {
                        if y > 0.0 {
                            translate_y -= 0.01;
                        } else {
                            translate_y += 0.01;
                        }
                        if x > 0.0 {
                            translate_x -= 0.01;
                        } else {
                            translate_x += 0.01;
                        }
                    }
                }
            }
            Event::DeviceEvent { event: DeviceEvent::MouseWheel { delta, .. }, .. } => {
                match delta {
                    MouseScrollDelta::LineDelta(_x, y) => {
                        if zoom_flag {
                            if y > 0.0 {
                                zoom += 0.01;
                            } else {
                                if zoom > 0.02 {
                                    zoom -= 0.01;
                                }
                            }
                        }
                    }
                    MouseScrollDelta::PixelDelta(_) => {}
                }
            }
            Event::RedrawEventsCleared => {
                previous_frame_end.as_mut().unwrap().cleanup_finished();

                if recreate_swapchain {
                    let dimensions: [u32; 2] = surface.window().inner_size().into();
                    let (new_swapchain, new_images) = match swapchain.recreate_with_dimensions(dimensions) {
                        Ok(r) => r,
                        Err(SwapchainCreationError::UnsupportedDimensions) => return,
                        Err(e) => panic!("Failed to recreate swapchain: {:?}", e)
                    };

                    swapchain = new_swapchain;
                    let (new_pipeline, new_framebuffers) = window_size_dependent_setup(device.clone(), &vs, &fs, &new_images, render_pass.clone());
                    pipeline = new_pipeline;
                    framebuffers = new_framebuffers;
                    recreate_swapchain = false;
                }

                let (image_num, suboptimal, acquire_future) = match swapchain::acquire_next_image(swapchain.clone(), None) {
                    Ok(r) => r,
                    Err(AcquireError::OutOfDate) => {
                        recreate_swapchain = true;
                        return;
                    }
                    Err(e) => panic!("Failed to acquire next image: {:?}", e)
                };


                let uniform_buffer_subbuffer = {
                    let elapsed = rotation_start.elapsed();
                    let rotation = elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 / 1_000_000_000.0;
                    let rotation = Matrix3::from_angle_y(Rad(rotation as f32));

                    // note: this teapot was meant for OpenGL where the origin is at the lower left
                    //       instead the origin is at the upper left in Vulkan, so we reverse the Y axis
                    let aspect_ratio = dimensions[0] as f32 / dimensions[1] as f32;
                    let proj = cgmath::perspective(Rad(std::f32::consts::FRAC_PI_2), aspect_ratio, 0.01, 100.0);
                    let view = Matrix4::look_at(Point3::new(0.3, 0.3, 1.0), Point3::new(0.0, 0.0, 0.0), Vector3::new(0.0, -1.0, 0.0));
                    let scale = Matrix4::from_scale(zoom);
                    let translate = Matrix4::from_translation(Vector3::new(translate_x, translate_y, 0.0));
                    //uniforms and Index is material specified parameter so 


                    let uniform_data = vs::ty::Data {
                        world: Matrix4::from(rotation).into(),
                        view: (view * translate * scale).into(),
                        proj: proj.into(),
                    };

                    uniform_buffer.next(uniform_data).unwrap()
                };

                if suboptimal {
                    recreate_swapchain = true;
                }


                let mut builder = AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family()).unwrap();
                    builder.begin_render_pass(
                        framebuffers[image_num].clone(), false,
                        vec![
                            [0.0, 0.0, 1.0, 1.0].into(),
                            1f32.into()
                        ],
                    ).unwrap();
//マテリアルに応じたテクスチャ、環境光、拡散光、鏡面光を設定
                for asset in &assets {
                    let texture_id = asset.texture;
                    let index_buffer = asset.ibo.clone();
                    let diffuse = asset.diffuse;
                    let specular = asset.specular;
                    let specular_intensity = asset.specular_intensity;
                    let ambient = asset.ambient;
                    let edge_flag = asset.edge_flag as u32;
                    let edge_color = asset.edge_color;

                    let uniform_data_material = fs::ty::Material {
                        diffuse,
                        specular,
                        specular_intensity,
                        ambient,
                        edge_color,
                        edge_flag,
                    };

                    let uniform_buffer_subbuffer_image = {
                        let dim = textures[texture_id].dimensions();
                        let image_w = dim.width() as f32;
                        let image_h = dim.height() as f32;
                        let uniform_data = fs::ty::Image {
                            w: image_w,
                            h: image_h,
                        };
                        uniform_buffer_screen.next(uniform_data).unwrap()
                    };
                    let uniform_buffer_subbuffer_material = uniform_buffer_material.next(uniform_data_material).unwrap();
                    let layout = pipeline.descriptor_set_layout(0).unwrap();
                    let layout1 = pipeline.descriptor_set_layout(1).unwrap();

                    let set = Arc::new(PersistentDescriptorSet::start(layout.clone())
                        .add_buffer(uniform_buffer_subbuffer.clone()).unwrap()
                        .add_sampled_image(textures[texture_id].clone(), sampler.clone()).unwrap()
                        .add_buffer(uniform_buffer_subbuffer_image.clone()).unwrap()
                        .build().unwrap()
                    );
                    let set1 = Arc::new(
                        PersistentDescriptorSet::start(layout1.clone())
                            .add_buffer(uniform_buffer_subbuffer_material.clone()).unwrap()
                            .build().unwrap()
                    );
                     builder.draw_indexed(
                        pipeline.clone(),
                        &DynamicState::none(),
                        vec!(vertex_buffer.clone()),
                        index_buffer.clone(), (set, set1), ()).unwrap();
                }
                builder.end_render_pass().unwrap();
                let command_buffer =    builder.build().unwrap();


                let future = previous_frame_end.take().unwrap().join(acquire_future)
                    .then_execute(queue.clone(), command_buffer).unwrap()
                    .then_swapchain_present(queue.clone(), swapchain.clone(), image_num)
                    .then_signal_fence_and_flush();

                match future {
                    Ok(future) => {
                        previous_frame_end = Some(Box::new(future) as Box<_>);
                    }
                    Err(FlushError::OutOfDate) => {
                        recreate_swapchain = true;
                        previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<_>);
                    }
                    Err(e) => {
                        println!("Failed to flush future: {:?}", e);
                        previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<_>);
                    }
                }
            }
            _ => ()
        }
    });
}

/// This method is called once during initialization, then again whenever the window is resized
fn window_size_dependent_setup(
    device: Arc<Device>,
    vs: &vs::Shader,
    fs: &fs::Shader,
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
) -> (Arc<dyn GraphicsPipelineAbstract + Send + Sync>, Vec<Arc<dyn FramebufferAbstract + Send + Sync>>) {
    let dimensions = images[0].dimensions();

    let depth_buffer = AttachmentImage::transient(device.clone(), dimensions, Format::D32Sfloat).unwrap();

    let framebuffers = images.iter().map(|image| {
        Arc::new(
            Framebuffer::start(render_pass.clone())
                .add(image.clone()).unwrap()
                .add(depth_buffer.clone()).unwrap()
                .build().unwrap()
        ) as Arc<dyn FramebufferAbstract + Send + Sync>
    }).collect::<Vec<_>>();

    // In the triangle example we use a dynamic viewport, as its a simple example.
    // However in the teapot example, we recreate the pipelines with a hardcoded viewport instead.
    // This allows the driver to optimize things, at the cost of slower window resizes.
    // https://computergraphics.stackexchange.com/questions/5742/vulkan-best-way-of-updating-pipeline-viewport
    let pipeline = Arc::new(GraphicsPipeline::start()
        .vertex_input(SingleBufferDefinition::<Vertex>::new())
        .vertex_shader(vs.main_entry_point(), ())
        .triangle_list()
        .viewports_dynamic_scissors_irrelevant(1)
        .viewports(iter::once(Viewport {
            origin: [0.0, 0.0],
            dimensions: [dimensions[0] as f32, dimensions[1] as f32],
            depth_range: 0.0..1.0,
        }))
        .fragment_shader(fs.main_entry_point(), ())
        .depth_stencil_simple_depth()
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        .build(device.clone())
        .unwrap());

    (pipeline, framebuffers)
}

