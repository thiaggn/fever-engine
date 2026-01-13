#![allow(unused)]

use pollster::FutureExt;
use wgpu::{InstanceDescriptor, RequestAdapterOptions, include_wgsl};

use crate::window::NativeHandle;

pub struct Renderer {
    surface: wgpu::Surface<'static>,
}

impl Renderer {
    pub fn new(window_handle: NativeHandle) -> Self {
        // ponto de entrada para criação de `surface` e `instance`
        let instance = wgpu::Instance::new(&InstanceDescriptor::default());

        // `surface` é a superficie de renderização
        let surface = instance
            .create_surface(Box::new(window_handle))
            .expect("surface deve ser criado na inicialização do renderer");

        // `adapter` é a gpu escolhida para renderização
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .block_on()
            .expect("falhou em encontrar um adapter apropriado.");

        // `device` é a gpu física
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .block_on()
            .expect("falhou em estabelecer integração com a GPU");

        // o shader será salvo em tempo de compilação
        let shader = device.create_shader_module(include_wgsl!("../../resources/shader.wgsl"));

        let pip_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            immediate_size: 0,
        });

        let swapchain_capab = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capab.formats[0];

        let render_pip = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pip_layout),
            depth_stencil: None,
            multiview_mask: None,
            cache: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(swapchain_format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            multisample: wgpu::MultisampleState::default(),
        });

        Self { surface }
    }

    pub fn render(&mut self) {}
}
