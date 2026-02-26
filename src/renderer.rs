use crate::input::InputState;
use pollster::FutureExt;
use std::{borrow::Cow, sync::Arc};
use winit::{dpi::PhysicalSize, window::Window};

/// Formato de cores do frame buffer preferido pelo sistema de renderização.
const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

/// Reune o estado global do sistema de renderização.
pub struct RenderSystem {
	device: wgpu::Device,
	adapter: wgpu::Adapter,
	queue: wgpu::Queue,
	instance: wgpu::Instance,
	renderers: RendererVec,
}

pub struct RenderSurface {
	inner: wgpu::Surface<'static>,
	config: wgpu::SurfaceConfiguration,
}

impl RenderSystem {
	pub fn new() -> Self {
		let instance = wgpu::Instance::new(&Default::default());

		let adapter = instance
			.request_adapter(&wgpu::RequestAdapterOptions {
				..Default::default()
			})
			.block_on()
			.expect("wgpu: a criação do adapter falhou.");

		let (device, queue) = adapter
			.request_device(&wgpu::DeviceDescriptor {
				label: None,
				..Default::default()
			})
			.block_on()
			.expect("wgpu: a criação do device falhou.");

		let mut renderers = RendererVec::new();
		renderers.add(TriangleRenderer::new(&device, TEXTURE_FORMAT));

		Self {
			device,
			queue,
			instance,
			adapter,
			renderers,
		}
	}

	pub fn create_surface(&self, target: Arc<Window>) -> RenderSurface {
		let PhysicalSize { width, height } = target.inner_size();

		let surface = self
			.instance
			.create_surface(target)
			.expect("wgpu: falhou em criar superfície para o alvo fornecido.");

		let config = surface
			.get_default_config(&self.adapter, width, height)
			.expect("wgpu: falhou em obter a configuração da superfície.");

		surface.configure(&self.device, &config);

		RenderSurface {
			inner: surface,
			config,
		}
	}

	pub fn configure_size(&self, surface: &mut RenderSurface, w: u32, h: u32) {
		surface.config.width = w;
		surface.config.height = h;
		surface.inner.configure(&self.device, &surface.config);
	}

	pub fn draw(&mut self, surface: &RenderSurface) {
		let frame = surface
			.inner
			.get_current_texture()
			.expect("wgpu: falhou em obter a próxima textura do swapchain.");

		let view = frame
			.texture
			.create_view(&wgpu::TextureViewDescriptor::default());

		let mut encoder = self
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

		let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: None,
			color_attachments: &[Some(wgpu::RenderPassColorAttachment {
				view: &view,
				depth_slice: None,
				resolve_target: None,
				ops: wgpu::Operations {
					load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
					store: wgpu::StoreOp::Store,
				},
			})],
			depth_stencil_attachment: None,
			timestamp_writes: None,
			occlusion_query_set: None,
			multiview_mask: None,
		});

		for renderer in self.renderers.iter_mut() {
			renderer.prepare();
			renderer.render(&mut render_pass);
		}

		render_pass.forget_lifetime();
		self.queue.submit(Some(encoder.finish()));
		frame.present();
	}
}

/// Um contrato para componentes renderizáveis no pipeline.
trait Renderer {
	/// Prepara recursos em tempo de execução, a cada passe.
	fn prepare(&mut self);

	/// Efetua a renderização.
	fn render(&self, pass: &mut wgpu::RenderPass);
}

struct RendererVec {
	items: Vec<Box<dyn Renderer>>,
}

impl RendererVec {
	pub fn new() -> Self {
		Self { items: Vec::new() }
	}

	pub fn add<R: Renderer + 'static>(&mut self, renderer: R) {
		self.items.push(Box::new(renderer));
	}

	pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Box<dyn Renderer>> {
		self.items.iter_mut()
	}
}

struct TriangleRenderer {
	pipeline: wgpu::RenderPipeline,
}

impl TriangleRenderer {
	fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
		let shader =
			device.create_shader_module(wgpu::include_wgsl!("../assets/shader.wgsl"));

		let pipeline_layout =
			device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: None,
				bind_group_layouts: &[],
				immediate_size: 0,
			});

		let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: None,
			layout: Some(&pipeline_layout),
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
				targets: &[Some(format.into())],
			}),
			primitive: wgpu::PrimitiveState::default(),
			depth_stencil: None,
			multisample: wgpu::MultisampleState::default(),
			multiview_mask: None,
			cache: None,
		});

		return Self { pipeline };
	}
}

impl Renderer for TriangleRenderer {
	fn prepare(&mut self) {}

	fn render(&self, pass: &mut wgpu::RenderPass) {
		pass.set_pipeline(&self.pipeline);
		pass.draw(0..3, 0..1);
	}
}
