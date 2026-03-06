use crate::cube::CubeRenderer;
use pollster::FutureExt;
use std::sync::Arc;
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
				required_features: wgpu::Features::POLYGON_MODE_LINE,
				..Default::default()
			})
			.block_on()
			.expect("wgpu: a criação do device falhou.");

		let context = RenderContext {
			device: &device,
			queue: &queue,
			format: TEXTURE_FORMAT,
		};

		let mut renderers = RendererVec::new();
		renderers.add(CubeRenderer::new(context));

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
					load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
					store: wgpu::StoreOp::Store,
				},
			})],
			depth_stencil_attachment: None,
			timestamp_writes: None,
			occlusion_query_set: None,
			multiview_mask: None,
		});

		for r in self.renderers.iter_mut() {
			r.prepare();
			r.render(&mut render_pass);
		}

		render_pass.forget_lifetime();
		self.queue.submit(Some(encoder.finish()));
		frame.present();
	}
}

pub struct RenderContext<'a> {
	pub device: &'a wgpu::Device,
	pub queue: &'a wgpu::Queue,
	pub format: wgpu::TextureFormat,
}

/// Um contrato para componentes renderizáveis no pipeline.
pub trait Renderer {
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
