#![feature(test)]

mod resource;

use vulkano_win::VkSurfaceBuild;

use log::{debug, error, info, warn};
use vulkano::{impl_vertex, single_pass_renderpass, ordered_passes_renderpass};
use vulkano::sync::GpuFuture;
use vulkano::instance::debug::{self, DebugCallback};
use cgmath::prelude::*;

use spin_sleep::LoopHelper;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;
const FOV: f32 = 90.0;

pub fn run() {
    info!("Initializing Vulkan");
    debug!("Creating Vulkan instance");
    let instance = {
        let extensions = vulkano::instance::InstanceExtensions {
            ext_debug_report: true,
            ..vulkano_win::required_extensions()
        };
        vulkano::instance::Instance::new(None, &extensions, vec!["VK_LAYER_LUNARG_standard_validation"]).expect("failed to create Vulkan instance")
    };

    let _debug_callback = DebugCallback::new(&instance, debug::MessageTypes {
        error: true,
        warning: true,
        performance_warning: true,
        information: false,
        debug: false,
    }, move |msg| {
        let debug_msg = &format!("{}: {}", msg.layer_prefix, msg.description);
        if msg.ty.error {
            error!("{}", debug_msg);
        } else if msg.ty.warning || msg.ty.performance_warning {
            warn!("{}", debug_msg);
        } else if msg.ty.information {
            info!("{}", debug_msg);
        } else if msg.ty.debug {
            debug!("{}", debug_msg);
        } else {
            unreachable!();
        }
    }).expect("failed to create debug callback");

    debug!("Selecting physical device");
    let physical = vulkano::instance::PhysicalDevice::enumerate(&instance)
        .next().expect("no device available");
    info!("Using device: {} (type: {:?})", physical.name(), physical.ty());

    debug!("Initializing window");
    let mut events_loop = winit::EventsLoop::new();
    let surface = winit::WindowBuilder::new()
        .with_dimensions(winit::dpi::LogicalSize::new(WIDTH.into(), HEIGHT.into()))
        .with_resizable(false)
        .with_title("Rusty RPG")
        .build_vk_surface(&events_loop, instance.clone()).unwrap();
    
    debug!("Selecting GPU");
    let queue_family = physical.queue_families().find(|&q| {
        q.supports_graphics() && surface.is_supported(q).unwrap_or(false)
    }).expect("couldn't find a graphical queue family");

    debug!("Initializing device");
    let (device, mut queues) = {
        let device_ext = vulkano::device::DeviceExtensions {
            khr_swapchain: true,
            .. vulkano::device::DeviceExtensions::none()
        };

        vulkano::device::Device::new(physical, physical.supported_features(), &device_ext,
            [(queue_family, 0.5)].iter().cloned()).expect("failed to create device")
    };

    let queue = queues.next().unwrap();

    let mut dimensions;

    debug!("Initializing swapchain");
    let (mut swapchain, mut images) = {
        let caps = surface.capabilities(physical).expect("failed to get surface capabilities");
        dimensions = caps.current_extent.unwrap_or([WIDTH, HEIGHT]);

        let format = caps.supported_formats[0].0;

        let present_mode = if caps.present_modes.supports(vulkano::swapchain::PresentMode::Mailbox) {
            vulkano::swapchain::PresentMode::Mailbox
        } else if caps.present_modes.supports(vulkano::swapchain::PresentMode::Immediate) {
            vulkano::swapchain::PresentMode::Immediate
        } else {
            vulkano::swapchain::PresentMode::Fifo
        };
        debug!("Present mode: {:?}", present_mode);

        let image_count = if let Some(max_images) = caps.max_image_count {
            u32::min(caps.min_image_count + 1, max_images)
        } else {
            caps.min_image_count + 1
        };

        vulkano::swapchain::Swapchain::new(
            device.clone(),
            surface.clone(),
            image_count,
            format,
            dimensions,
            1,
            caps.supported_usage_flags,
            &queue,
            vulkano::swapchain::SurfaceTransform::Identity,
            vulkano::swapchain::CompositeAlpha::Opaque,
            present_mode,
            true,
            None
        ).expect("failed to create swapchain")
    };

    debug!("Initializing depth buffer");
    let mut depth_buffer = vulkano::image::attachment::AttachmentImage::transient(device.clone(), dimensions, vulkano::format::D16Unorm).expect("failed to create depth buffer");

    debug!("Initializing vertex buffer");
    let vertex_buffer = {
        #[derive(Debug, Clone)]
        struct Vertex {
            position: [f32; 3], 
            color: [f32; 3],
            tex_coord: [f32; 2],
        }
        impl_vertex!(Vertex, position, color, tex_coord);

        vulkano::buffer::CpuAccessibleBuffer::from_iter(
            device.clone(), vulkano::buffer::BufferUsage::all(), [
                Vertex { position: [-0.5, -0.5, 0.0],   color: [1.0, 0.0, 0.0], tex_coord: [0.0, 0.0] },
                Vertex { position: [0.5, -0.5, 0.0],    color: [0.0, 1.0, 0.0], tex_coord: [1.0, 0.0] },
                Vertex { position: [0.5, 0.5, 0.0],     color: [0.0, 0.0, 1.0], tex_coord: [1.0, 1.0] },
                Vertex { position: [-0.5, 0.5, 0.0],    color: [1.0, 1.0, 1.0], tex_coord: [0.0, 1.0] },
            ].iter().cloned()).expect("failed to create buffer")
    };

    debug!("Initializing index buffer");
    let index_buffer = vulkano::buffer::cpu_access::CpuAccessibleBuffer::from_iter(
        device.clone(), vulkano::buffer::BufferUsage::all(), ([
            0, 1, 2,
            2, 3, 0,
        ] as [u16; 6]).iter().cloned()).expect("failed to create buffer");

    debug!("Initializing shaders");
    mod vs {
        use vulkano_shader_derive::VulkanoShader;
        #[derive(VulkanoShader)]
        #[ty = "vertex"]
        #[path = "../resources/shaders/sprite.vert"]
        #[allow(dead_code)]
        struct Dummy;
    }

    mod fs {
        use vulkano_shader_derive::VulkanoShader;
        #[derive(VulkanoShader)]
        #[ty = "fragment"]
        #[path = "../resources/shaders/sprite.frag"]
        #[allow(dead_code)]
        struct Dummy;
    }

    let uniform_buffer = vulkano::buffer::cpu_pool::CpuBufferPool::<vs::ty::Data>::new(device.clone(), vulkano::buffer::BufferUsage::all());

    let vs = vs::Shader::load(device.clone()).expect("failed to create shader module");
    let fs = fs::Shader::load(device.clone()).expect("failed to create shader module");

    debug!("Creating render pass");
    let render_pass = std::sync::Arc::new(single_pass_renderpass!(device.clone(),
        attachments: {
            color: {
                load: Clear, // clear the content of this attachment at the start of the drawing
                store: Store, // store the output of the draw in the actual image
                format: swapchain.format(),
                samples: 1,
            },
            depth: {
                load: Clear,
                store: DontCare,
                format: vulkano::format::Format::D16Unorm,
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {depth}
        }
    ).unwrap());

    debug!("Loading resources");
    let mut resource_manager = crate::resource::ResourceManager::new();
    let resource_future = resource_manager.load_resources(&device, &queue);

    debug!("Creating sampler");
    let sampler = vulkano::sampler::Sampler::new(
        device.clone(),
        vulkano::sampler::Filter::Linear,
        vulkano::sampler::Filter::Linear,
        vulkano::sampler::MipmapMode::Nearest,
        vulkano::sampler::SamplerAddressMode::Repeat,
        vulkano::sampler::SamplerAddressMode::Repeat,
        vulkano::sampler::SamplerAddressMode::Repeat,
        0.0, 1.0, 0.0, 0.0).expect("failed to create sampler");

    debug!("Creating pipeline");
    let pipeline = std::sync::Arc::new(vulkano::pipeline::GraphicsPipeline::start()
        .vertex_input_single_buffer()
        .vertex_shader(vs.main_entry_point(), ())
        .triangle_list()
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs.main_entry_point(), ())
        .depth_stencil_simple_depth()
        // .blend_alpha_blending()
        .render_pass(vulkano::framebuffer::Subpass::from(render_pass.clone(), 0).unwrap())
        .build(device.clone())
        .unwrap());
    
    debug!("Creating persistent descriptor set");
    let persistent_set = std::sync::Arc::new(vulkano::descriptor::descriptor_set::PersistentDescriptorSet::start(pipeline.clone(), 0)
        .add_sampled_image(resource_manager.get_texture(crate::resource::ResourceID::from("awesome_face")), sampler.clone()).unwrap()
        .build().unwrap()
    );
    
    debug!("Creating framebuffers");
    let mut framebuffers: Option<Vec<std::sync::Arc<vulkano::framebuffer::Framebuffer<_,_>>>> = None;
    
    let mut recreate_swapchain = false;

    let mut previous_frame_end = resource_future;

    debug!("Creating dynamic state");
    let mut dynamic_state = vulkano::command_buffer::DynamicState {
        line_width: None,
        viewports: Some(vec![vulkano::pipeline::viewport::Viewport {
            origin: [0.0, 0.0],
            dimensions: [dimensions[0] as f32, dimensions[1] as f32],
            depth_range: 0.0 .. 1.0,
        }]),
        scissors: None,
    };

    let mut loop_helper = LoopHelper::builder()
        .report_interval_s(0.5)
        .build_with_target_rate(250.0);

    info!("Starting main loop");

    loop {
        let _delta = loop_helper.loop_start();

        // Create transformations
        let projection = cgmath::perspective(cgmath::Deg(FOV), WIDTH as f32 / HEIGHT as f32, 0.1, 100.0);
        // let projection = cgmath::ortho(-10.0, 10.0, -10.0, 10.0, 0.0, 100.0);
        let view = cgmath::Matrix4::look_at(cgmath::Point3::new(0., 0., 4.),
                                            cgmath::Point3::new(0., 0., 0.),
                                            cgmath::vec3(0., 1., 0.));
        let world: cgmath::Matrix4<f32> = cgmath::Matrix4::identity();

        previous_frame_end.cleanup_finished();

        if recreate_swapchain {
            dimensions = surface.capabilities(physical)
                .expect("failed to get surface capabilities")
                .current_extent.unwrap();
            
            let (new_swapchain, new_images) = match swapchain.recreate_with_dimension(dimensions) {
                Ok(r) => r,
                Err(vulkano::swapchain::SwapchainCreationError::UnsupportedDimensions) => {
                    continue;
                },
                Err(err) => panic!("{:?}", err)
            };

            swapchain = new_swapchain;
            images = new_images;

            depth_buffer = vulkano::image::attachment::AttachmentImage::transient(device.clone(), dimensions, vulkano::format::D16Unorm).expect("failed to create depth buffer");

            framebuffers = None;

            dynamic_state.viewports = Some(vec![vulkano::pipeline::viewport::Viewport {
                origin: [0.0, 0.0],
                dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                depth_range: 0.0 .. 1.0,
            }]);

            recreate_swapchain = false;
        }

        if framebuffers.is_none() {
            framebuffers = Some(images.iter().map(|image| {
                std::sync::Arc::new(vulkano::framebuffer::Framebuffer::start(render_pass.clone())
                    .add(image.clone()).unwrap()
                    .add(depth_buffer.clone()).unwrap()
                    .build().unwrap())
            }).collect::<Vec<_>>());
        }

        let uniform_buffer_subbuffer = {
            let uniform_data = vs::ty::Data {
                projection: projection.into(),
                view: view.into(),
                world: world.into(),
            };

            uniform_buffer.next(uniform_data).unwrap()
        };

        let (image_num, acquire_future) = match vulkano::swapchain::acquire_next_image(swapchain.clone(), None) {
            Ok(r) => r,
            Err(vulkano::swapchain::AcquireError::OutOfDate) => {
                recreate_swapchain = true;
                continue;
            },
            Err(err) => panic!("{:?}", err)
        };

        let mutable_set = std::sync::Arc::new(vulkano::descriptor::descriptor_set::PersistentDescriptorSet::start(pipeline.clone(), 1)
            .add_buffer(uniform_buffer_subbuffer).unwrap()
            .build().unwrap()
        );

        let command_buffer = vulkano::command_buffer::AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family()).unwrap()
            .begin_render_pass(framebuffers.as_ref().unwrap()[image_num].clone(), false,
                vec![[0.0, 0.0, 0.0, 1.0].into(), 1f32.into()])
            .unwrap()
            .draw_indexed(
                pipeline.clone(),
                &dynamic_state,
                vertex_buffer.clone(),
                index_buffer.clone(),
                (persistent_set.clone(), mutable_set.clone()),
                ()).unwrap()
            .end_render_pass().unwrap()
            .build().unwrap();
        
        let future = previous_frame_end.join(acquire_future)
            .then_execute(queue.clone(), command_buffer).unwrap()
            .then_swapchain_present(queue.clone(), swapchain.clone(), image_num)
            .then_signal_fence_and_flush();
        
        match future {
            Ok(future) => {
                previous_frame_end = Box::new(future) as Box<_>;
            },
            Err(vulkano::sync::FlushError::OutOfDate) => {
                recreate_swapchain = true;
                previous_frame_end = Box::new(vulkano::sync::now(device.clone())) as Box<_>;
            },
            Err(e) => {
                error!("{:?}", e);
                previous_frame_end = Box::new(vulkano::sync::now(device.clone())) as Box<_>;
            }
        }

        if let Some(fps) = loop_helper.report_rate() {
            if fps < 30.0 {
                warn!("Low FPS: {}", fps.round());
            }
            surface.window().set_title(&format!("Rusty RPG: {} FPS", fps.round()));
        }

        let mut done = false;
        events_loop.poll_events(|ev| {
            match ev {
                winit::Event::WindowEvent { event: winit::WindowEvent::CloseRequested, .. } => done = true,
                winit::Event::WindowEvent { event: winit::WindowEvent::KeyboardInput{ input: winit::KeyboardInput{ virtual_keycode: Some(winit::VirtualKeyCode::Escape), .. }, .. }, .. } => done = true,
                _ => ()
            }
        });
        if done { return; }
        
        loop_helper.loop_sleep();
    }
}