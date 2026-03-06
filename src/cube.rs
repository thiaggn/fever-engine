#![allow(unused)]

use std::{borrow::Cow, num::NonZero};

use glam::Mat4;
use image::{EncodableLayout, ImageReader};
use wgpu::{
	BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
	BindGroupLayoutEntry, BindingResource, BindingType, BufferBindingType, BufferSize,
	BufferUsages, FragmentState, IndexFormat, PipelineLayoutDescriptor, PrimitiveState,
	RenderPipeline, RenderPipelineDescriptor, ShaderStages, VertexAttribute, VertexBufferLayout,
	VertexState, VertexStepMode,
	util::{BufferInitDescriptor, DeviceExt, RenderEncoder},
};

use crate::{
	math,
	renderer::{RenderContext, Renderer},
};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GlobalUniforms {
	view_proj: Mat4,
}

pub struct CubeRenderer {
	pipeline: RenderPipeline,
	vertex_buffer: wgpu::Buffer,
	index_buffer: wgpu::Buffer,
	bind_group: BindGroup,
	index_count: u32,
}

impl CubeRenderer {
	pub fn new(ctx: RenderContext) -> Self {
		#[rustfmt::skip]
		let vertices: [f32; 8*3] = [
			 1.0,  1.0,  1.0, // 0
			-1.0,  1.0,  1.0, // 1
			-1.0, -1.0,  1.0, // 2
			 1.0, -1.0,  1.0, // 3
			 1.0,  1.0, -1.0, // 4
			-1.0,  1.0, -1.0, // 5
			-1.0, -1.0, -1.0, // 6
			 1.0, -1.0, -1.0, // 7
		];

		let vertex_layout = VertexBufferLayout {
			array_stride: 12,
			step_mode: VertexStepMode::Vertex,
			attributes: &[VertexAttribute {
				format: wgpu::VertexFormat::Float32x3,
				offset: 0,
				shader_location: 0, // @location(0)
			}],
		};

		let vertex_buffer = ctx.device.create_buffer_init(&BufferInitDescriptor {
			label: Some("cube vertex buffer"),
			contents: bytemuck::cast_slice(&vertices),
			usage: BufferUsages::VERTEX,
		});

		#[rustfmt::skip]
		let indices: [u16; 6*6] = [
			0, 1, 2, 2, 3, 0, // frente
			5, 4, 7, 7, 6, 5, // trás
			4, 0, 3, 3, 7, 4, // direita
			1, 5, 6, 6, 2, 1, // esquerda
			4, 5, 1, 1, 0, 4, // topo
			3, 2, 6, 6, 7, 3, // fundo
		];

		let index_buffer = ctx.device.create_buffer_init(&BufferInitDescriptor {
			label: Some("cube index buffer"),
			contents: bytemuck::cast_slice(&indices),
			usage: BufferUsages::INDEX,
		});

		let view = Mat4::look_at_rh(
			glam::Vec3::new(-2.0, -2.0, 3.0), // posição da câmera (afastada 5 unidades)
			glam::Vec3::ZERO,                 // ponto que a câmera olha (centro do cubo)
			glam::Vec3::Y,                    // vetor "up"
		);

		let globals = GlobalUniforms {
			view_proj: math::perspective(1.0, 20.0, 90.0, 16.0 / 9.0) * view,
		};

		let globals_buffer = ctx.device.create_buffer_init(&BufferInitDescriptor {
			label: Some("cube global uniforms"),
			contents: bytemuck::bytes_of(&globals),
			usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
		});

		let bind_group_layout = ctx
			.device
			.create_bind_group_layout(&BindGroupLayoutDescriptor {
				label: Some("cube bind group layout"),
				entries: &[BindGroupLayoutEntry {
					binding: 0,
					visibility: ShaderStages::VERTEX, // usamos a matriz de projeção só no vertex shader
					count: None,
					ty: BindingType::Buffer {
						ty: BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None,
					},
				}],
			});

		let bind_group = ctx.device.create_bind_group(&BindGroupDescriptor {
			label: Some("cube bind group"),
			layout: &bind_group_layout,
			entries: &[BindGroupEntry {
				binding: 0,
				resource: globals_buffer.as_entire_binding(),
			}],
		});

		let shader_module = ctx
			.device
			.create_shader_module(wgpu::include_wgsl!("../assets/shader.wgsl"));

		let pipeline_layout = ctx
			.device
			.create_pipeline_layout(&PipelineLayoutDescriptor {
				label: None,
				bind_group_layouts: &[&bind_group_layout],
				immediate_size: 0,
			});

		let pipeline = ctx
			.device
			.create_render_pipeline(&RenderPipelineDescriptor {
				label: None,
				layout: Some(&pipeline_layout),
				vertex: VertexState {
					module: &shader_module,
					entry_point: Some("main_vertex"),
					compilation_options: Default::default(),
					buffers: &[vertex_layout],
				},
				fragment: Some(FragmentState {
					module: &shader_module,
					entry_point: Some("main_frag"),
					compilation_options: Default::default(),
					targets: &[Some(ctx.format.into())],
				}),
				primitive: PrimitiveState {
					topology: wgpu::PrimitiveTopology::TriangleList,
					polygon_mode: wgpu::PolygonMode::Fill,
					..Default::default()
				},
				multisample: Default::default(),
				multiview_mask: None,
				depth_stencil: None,
				cache: None,
			});

		Self {
			pipeline,
			vertex_buffer,
			index_buffer,
			bind_group,
			index_count: indices.len() as u32,
		}
	}
}

impl Renderer for CubeRenderer {
	fn prepare(&mut self) {}

	fn render(&self, pass: &mut wgpu::RenderPass) {
		pass.set_pipeline(&self.pipeline);
		pass.set_bind_group(0, &self.bind_group, &[]);
		pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
		pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
		pass.draw_indexed(0..self.index_count, 0, 0..1);
	}
}
