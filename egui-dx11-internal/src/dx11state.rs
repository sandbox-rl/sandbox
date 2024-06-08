use parking_lot::Mutex;
use windows::core::s;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Direct3D11::{
	ID3D11Device, ID3D11InputLayout, ID3D11RenderTargetView, ID3D11Texture2D,
	D3D11_APPEND_ALIGNED_ELEMENT, D3D11_INPUT_ELEMENT_DESC, D3D11_INPUT_PER_VERTEX_DATA,
	D3D11_VIEWPORT,
};
use windows::Win32::Graphics::Dxgi::Common::{
	DXGI_FORMAT_R32G32B32A32_FLOAT, DXGI_FORMAT_R32G32_FLOAT,
};
use windows::Win32::Graphics::Dxgi::{IDXGISwapChain, DXGI_SWAP_CHAIN_DESC};
use windows::Win32::UI::WindowsAndMessaging::GetClientRect;

use crate::backup::BackupState;
use crate::input::InputCollector;
use crate::shader::CompiledShaders;
use crate::texture::TextureAllocator;
use crate::WinResult;

pub struct Dx11State {
	pub input_layout: ID3D11InputLayout,
	pub shaders: CompiledShaders,
	pub tex_alloc: TextureAllocator,
	pub backup: BackupState,
	pub render_view: Mutex<ID3D11RenderTargetView>,
	pub input_collector: Mutex<InputCollector>,
	pub hwnd: HWND,
}

impl Dx11State {
	const INPUT_ELEMENTS_DESC: [D3D11_INPUT_ELEMENT_DESC; 3] = [
		D3D11_INPUT_ELEMENT_DESC {
			SemanticName: s!("POSITION"),
			SemanticIndex: 0,
			Format: DXGI_FORMAT_R32G32_FLOAT,
			InputSlot: 0,
			AlignedByteOffset: 0,
			InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
			InstanceDataStepRate: 0,
		},
		D3D11_INPUT_ELEMENT_DESC {
			SemanticName: s!("TEXCOORD"),
			SemanticIndex: 0,
			Format: DXGI_FORMAT_R32G32_FLOAT,
			InputSlot: 0,
			AlignedByteOffset: D3D11_APPEND_ALIGNED_ELEMENT,
			InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
			InstanceDataStepRate: 0,
		},
		D3D11_INPUT_ELEMENT_DESC {
			SemanticName: s!("COLOR"),
			SemanticIndex: 0,
			Format: DXGI_FORMAT_R32G32B32A32_FLOAT,
			InputSlot: 0,
			AlignedByteOffset: D3D11_APPEND_ALIGNED_ELEMENT,
			InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
			InstanceDataStepRate: 0,
		},
	];

	pub fn new(swapchain: &IDXGISwapChain) -> WinResult<Self> {
		let mut desc = DXGI_SWAP_CHAIN_DESC::default();
		let desc = unsafe { swapchain.GetDesc(&mut desc).map(|()| desc)? };

		let hwnd = desc.OutputWindow;

		let device: ID3D11Device = unsafe { swapchain.GetDevice() }?;

		let back_buffer: ID3D11Texture2D = unsafe { swapchain.GetBuffer(0) }?;

		let mut render_view = None;
		let render_view = unsafe {
			device
				.CreateRenderTargetView(&back_buffer, None, Some(&mut render_view))
				.map(|()| render_view.unwrap_unchecked())?
		};

		let shaders = CompiledShaders::new(&device)?;

		let mut input_layout = None;
		let input_layout = unsafe {
			device
				.CreateInputLayout(
					&Self::INPUT_ELEMENTS_DESC,
					&shaders.bytecode,
					Some(&mut input_layout),
				)
				.map(|()| input_layout.unwrap_unchecked())?
		};

		Ok(Dx11State {
			backup: BackupState::default(),
			input_collector: Mutex::new(InputCollector::new(hwnd)),
			tex_alloc: TextureAllocator::default(),
			render_view: Mutex::new(render_view),
			input_layout,
			shaders,
			hwnd,
		})
	}

	pub fn get_screen_size(&self) -> (f32, f32) {
		let mut rect = RECT::default();
		_ = unsafe { GetClientRect(self.hwnd, &mut rect) };
		(
			(rect.right - rect.left) as f32,
			(rect.bottom - rect.top) as f32,
		)
	}

	pub fn get_viewport(&self) -> D3D11_VIEWPORT {
		let (w, h) = self.get_screen_size();
		D3D11_VIEWPORT {
			TopLeftX: 0.0,
			TopLeftY: 0.0,
			Width: w,
			Height: h,
			MinDepth: 0.0,
			MaxDepth: 1.0,
		}
	}
}
