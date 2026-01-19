#![allow(unused)]

use pollster::FutureExt;
use wgpu::{InstanceDescriptor, RequestAdapterOptions, include_wgsl};

use crate::window::{self, NativeHandle};

pub struct Renderer {
	device: wgpu::Device,
	queue: wgpu::Queue,
	surface: wgpu::Surface<'static>,
	pipeline: wgpu::RenderPipeline,
	surface_cfg: wgpu::SurfaceConfiguration,
}

impl Renderer {
	pub fn new(window_handle: NativeHandle) -> Self {
		// Instance: descobre o ambiente gráfico disponível — “quais GPUs existem?”. Com
		// a instance iniciamos o wgpu, criamos a surface da janela e depois pedimos uma
		// GPU compatível com essa surface e com o backend escolhido.
		let instance = wgpu::Instance::new(&InstanceDescriptor::default());

		// Surface: onde a GPU vai desenhar. Faz a ligação entre a janela e a GPU e serve
		// como critério na escolha do adapter.
		let surface = instance
			.create_surface(Box::new(window_handle))
			.expect("surface deve ser criado na inicialização do renderer");

		// Adapter: seleciona e descreve uma GPU específica. Representa a GPU escolhida e
		// descreve o que ela suporta.
		let adapter = instance
			.request_adapter(&RequestAdapterOptions {
				compatible_surface: Some(&surface),
				..Default::default()
			})
			.block_on()
			.expect("falhou em encontrar um adapter apropriado.");

		// Device: conexão ativa com a GPU. A partir dele criamos recursos e enviamos
		// comandos para a GPU via queue.
		let (device, queue) = adapter
			.request_device(&wgpu::DeviceDescriptor::default())
			.block_on()
			.expect("falhou em estabelecer integração com a GPU");

		// o shader será salvo em tempo de compilação
		let shader =
			device.create_shader_module(include_wgsl!("../../resources/shader.wgsl"));

		let pipeline_layout =
			device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: None,
				bind_group_layouts: &[],
				immediate_size: 0,
			});

		let surface_capabilities = surface.get_capabilities(&adapter);
		let swapchain_format = surface_capabilities.formats[0];

		let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: None,
			layout: Some(&pipeline_layout),
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

		let surface_cfg = surface.get_default_config(&adapter, 1366, 768).unwrap();
		surface.configure(&device, &surface_cfg);

		Self {
			surface,
			surface_cfg,
			pipeline,
			device,
			queue,
		}
	}

	pub fn render(&self) {
		let frame = self
			.surface
			.get_current_texture()
			.expect("falhou em obter o próximo framebuffer do swapchain.");

		let mut view = frame
			.texture
			.create_view(&wgpu::TextureViewDescriptor::default());

		let mut encoder = self
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

		self.record_pass(&view, &mut encoder);
		self.queue.submit(Some(encoder.finish()));
		frame.present();
	}

	fn record_pass(&self, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) {
		let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			color_attachments: &[Some(wgpu::RenderPassColorAttachment {
				view: view,
				depth_slice: None,
				resolve_target: None,
				ops: wgpu::Operations {
					load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
					store: wgpu::StoreOp::Store,
				},
			})],
			..Default::default()
		});

		pass.set_pipeline(&self.pipeline);
		pass.draw(0..3, 0..1);
	}

	pub fn set_dimensions(&mut self, width: u32, height: u32) {
		self.surface_cfg.width = width;
		self.surface_cfg.height = height;
		self.surface.configure(&self.device, &self.surface_cfg);
	}
}
