use std::mem;
use std::sync::OnceLock;

use clipboard::windows_clipboard::WindowsClipboardContext;
use clipboard::ClipboardProvider;
use egui::epaint::Primitive;
use egui::Context;
use parking_lot::Mutex;
use windows::core::HRESULT;
use windows::Win32::Foundation::{FALSE, LPARAM, RECT, TRUE, WPARAM};
use windows::Win32::Graphics::Direct3D::D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST;
use windows::Win32::Graphics::Direct3D11::{
	ID3D11Device, ID3D11DeviceContext, ID3D11Texture2D, D3D11_BLEND_DESC,
	D3D11_BLEND_INV_SRC_ALPHA, D3D11_BLEND_ONE, D3D11_BLEND_OP_ADD, D3D11_BLEND_SRC_ALPHA,
	D3D11_COLOR_WRITE_ENABLE_ALL, D3D11_COMPARISON_ALWAYS, D3D11_CULL_NONE, D3D11_FILL_SOLID,
	D3D11_FILTER_MIN_MAG_MIP_LINEAR, D3D11_RASTERIZER_DESC, D3D11_RENDER_TARGET_BLEND_DESC,
	D3D11_SAMPLER_DESC, D3D11_TEXTURE_ADDRESS_BORDER,
};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_R32_UINT;
use windows::Win32::Graphics::Dxgi::IDXGISwapChain;

use crate::dx11state::Dx11State;
use crate::mesh::{self, GpuMesh, GpuVertex};
use crate::WinResult;

pub struct Dx11App<T, F>
where
	F: FnMut(&Context, &mut T) + Send + 'static,
{
	dx: OnceLock<Dx11State>,
	ui: Mutex<F>,
	ctx: Context,
	state: egui::mutex::Mutex<T>,
}

impl<T, F> Dx11App<T, F>
where
	F: FnMut(&Context, &mut T) + Send + 'static,
{
	pub fn new(ui: F, state: T) -> Self {
		Self {
			dx: OnceLock::new(),
			ui: Mutex::new(ui),
			ctx: Context::default(),
			state: egui::mutex::Mutex::new(state),
		}
	}

	fn set_blend_state(device: &ID3D11Device, context: &ID3D11DeviceContext) -> WinResult<()> {
		let targets: [D3D11_RENDER_TARGET_BLEND_DESC; 8] = [
			D3D11_RENDER_TARGET_BLEND_DESC {
				BlendEnable: TRUE,
				SrcBlend: D3D11_BLEND_SRC_ALPHA,
				DestBlend: D3D11_BLEND_INV_SRC_ALPHA,
				BlendOp: D3D11_BLEND_OP_ADD,
				SrcBlendAlpha: D3D11_BLEND_ONE,
				DestBlendAlpha: D3D11_BLEND_INV_SRC_ALPHA,
				BlendOpAlpha: D3D11_BLEND_OP_ADD,
				RenderTargetWriteMask: D3D11_COLOR_WRITE_ENABLE_ALL.0 as u8,
			},
			Default::default(),
			Default::default(),
			Default::default(),
			Default::default(),
			Default::default(),
			Default::default(),
			Default::default(),
		];

		let blend_desc = D3D11_BLEND_DESC {
			AlphaToCoverageEnable: FALSE,
			IndependentBlendEnable: FALSE,
			RenderTarget: targets,
		};

		let mut blend_state = None;
		let blend_state = unsafe {
			device
				.CreateBlendState(&blend_desc, Some(&mut blend_state))
				.map(|()| blend_state.unwrap_unchecked())?
		};

		unsafe { context.OMSetBlendState(&blend_state, Some(&[0., 0., 0., 0.]), 0xffffffff) };

		Ok(())
	}

	fn set_raster_options(device: &ID3D11Device, context: &ID3D11DeviceContext) -> WinResult<()> {
		let raster_desc = D3D11_RASTERIZER_DESC {
			FillMode: D3D11_FILL_SOLID,
			CullMode: D3D11_CULL_NONE,
			FrontCounterClockwise: FALSE,
			DepthBias: 0,
			DepthBiasClamp: 0.0,
			SlopeScaledDepthBias: 0.0,
			DepthClipEnable: FALSE,
			ScissorEnable: TRUE,
			MultisampleEnable: FALSE,
			AntialiasedLineEnable: FALSE,
		};

		let mut options = None;
		let options = unsafe {
			device
				.CreateRasterizerState(&raster_desc, Some(&mut options))
				.map(|()| options.unwrap_unchecked())?
		};

		unsafe { context.RSSetState(&options) };

		Ok(())
	}

	fn set_sampler_state(device: &ID3D11Device, context: &ID3D11DeviceContext) -> WinResult<()> {
		let desc = D3D11_SAMPLER_DESC {
			Filter: D3D11_FILTER_MIN_MAG_MIP_LINEAR,
			AddressU: D3D11_TEXTURE_ADDRESS_BORDER,
			AddressV: D3D11_TEXTURE_ADDRESS_BORDER,
			AddressW: D3D11_TEXTURE_ADDRESS_BORDER,
			MipLODBias: 0.0,
			ComparisonFunc: D3D11_COMPARISON_ALWAYS,
			MinLOD: 0.0,
			MaxLOD: 0.0,
			BorderColor: [1.0; 4],
			MaxAnisotropy: 0,
		};

		let mut sampler = None;
		let sampler = unsafe {
			device
				.CreateSamplerState(&desc, Some(&mut sampler))
				.map(|()| sampler.unwrap_unchecked())?
		};

		unsafe { context.PSSetSamplers(0, Some(&[Some(sampler)])) };

		Ok(())
	}

	pub fn present(&self, swapchain: &IDXGISwapChain) -> WinResult<()> {
		let dx @ Dx11State {
			tex_alloc,
			shaders,
			input_layout,
			backup,
			render_view,
			input_collector,
			..
		} = self.dx.get_or_try_init(|| Dx11State::new(swapchain))?;

		let device: ID3D11Device = unsafe { swapchain.GetDevice()? };
		let context = unsafe { device.GetImmediateContext() }?;

		backup.save(&context);

		let input = input_collector.lock().collect_input();

		let ui = &mut *self.ui.lock();

		let output = self
			.ctx
			.run(input, |ctx| (ui)(ctx, &mut *self.state.lock()));

		if !output.textures_delta.is_empty() {
			tex_alloc.process_deltas(&device, &context, output.textures_delta)?;
		}

		if !output.platform_output.copied_text.is_empty() {
			let _ = WindowsClipboardContext.set_contents(output.platform_output.copied_text);
		}

		let screen = dx.get_screen_size();

		let primitives = self
			.ctx
			.tessellate(output.shapes, self.ctx.pixels_per_point())
			.into_iter()
			.filter_map(|prim| {
				if let Primitive::Mesh(mesh) = prim.primitive {
					GpuMesh::from_mesh(screen, mesh, prim.clip_rect)
				} else {
					unimplemented!("Paint callbacks are not supported")
				}
			});

		Self::set_blend_state(&device, &context)?;
		Self::set_raster_options(&device, &context)?;
		Self::set_sampler_state(&device, &context)?;

		unsafe { context.RSSetViewports(Some(&[dx.get_viewport()])) };
		unsafe { context.OMSetRenderTargets(Some(&[Some(render_view.lock().clone())]), None) };
		unsafe { context.IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST) };
		unsafe { context.IASetInputLayout(input_layout) };

		for mesh in primitives {
			let idx = mesh::create_index_buffer(&device, &mesh)?;
			let vtx = mesh::create_vertex_buffer(&device, &mesh)?;

			let texture = tex_alloc.get_by_id(mesh.texture_id);

			unsafe {
				context.RSSetScissorRects(Some(&[RECT {
					left: mesh.clip.left() as _,
					top: mesh.clip.top() as _,
					right: mesh.clip.right() as _,
					bottom: mesh.clip.bottom() as _,
				}]))
			};

			if texture.is_some() {
				unsafe { context.PSSetShaderResources(0, Some(&[texture])) };
			}

			unsafe {
				context.IASetVertexBuffers(
					0,
					1,
					Some(&Some(vtx)),
					Some(&(mem::size_of::<GpuVertex>() as u32)),
					Some(&0),
				)
			};

			unsafe { context.IASetIndexBuffer(&idx, DXGI_FORMAT_R32_UINT, 0) };

			unsafe { context.VSSetShader(&shaders.vertex, Some(&[])) };
			unsafe { context.PSSetShader(&shaders.pixel, Some(&[])) };

			unsafe { context.DrawIndexed(mesh.indicies.len() as u32, 0, 0) };
		}

		backup.restore(&context);

		Ok(())
	}

	pub fn resize_buffers(
		&self,
		swapchain: &IDXGISwapChain,
		original: impl FnOnce() -> HRESULT,
	) -> WinResult<HRESULT> {
		let result = original();

		if let Some(dx) = self.dx.get() {
			let backbuffer: ID3D11Texture2D = unsafe { swapchain.GetBuffer(0) }?;

			let device: ID3D11Device = unsafe { swapchain.GetDevice() }?;

			let mut render_view = None;
			let render_view = unsafe {
				device
					.CreateRenderTargetView(&backbuffer, None, Some(&mut render_view))
					.map(|()| render_view.unwrap_unchecked())
			}?;

			*dx.render_view.lock() = render_view;
		}

		Ok(result)
	}

	pub fn wnd_proc(&self, umsg: u32, wparam: WPARAM, lparam: LPARAM) -> bool {
		if let Some(dx) = self.dx.get() {
			dx.input_collector.lock().process(umsg, wparam.0, lparam.0);
		}

		self.ctx.wants_pointer_input()
	}
}
