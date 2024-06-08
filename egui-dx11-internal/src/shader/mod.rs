use core::slice;

use windows::core::{s, PCSTR};
use windows::Win32::Graphics::Direct3D::Fxc::{D3DCompile, D3DCOMPILE_ENABLE_STRICTNESS};
use windows::Win32::Graphics::Direct3D::ID3DBlob;
use windows::Win32::Graphics::Direct3D11::{ID3D11Device, ID3D11PixelShader, ID3D11VertexShader};

pub struct CompiledShaders {
	pub vertex: ID3D11VertexShader,
	pub pixel: ID3D11PixelShader,
	pub bytecode: Vec<u8>,
}

impl CompiledShaders {
	pub fn new(device: &ID3D11Device) -> windows::core::Result<Self> {
		let vertex_blob = Self::compile_shader::<ID3D11VertexShader>()?;
		let pixel_blob = Self::compile_shader::<ID3D11PixelShader>()?;

		let bytecode = Self::bytecode(&vertex_blob).to_vec();

		let vertex = ID3D11VertexShader::create(device, &vertex_blob)?;
		let pixel = ID3D11PixelShader::create(device, &pixel_blob)?;

		Ok(Self {
			vertex,
			pixel,
			bytecode,
		})
	}

	fn bytecode(blob: &ID3DBlob) -> &[u8] {
		let ptr = unsafe { blob.GetBufferPointer() };
		let len = unsafe { blob.GetBufferSize() };

		unsafe { slice::from_raw_parts(ptr.cast(), len) }
	}

	fn compile_shader<S: Shader>() -> windows::core::Result<ID3DBlob> {
		const SHADER_TEXT: &str = include_str!("shader.hlsl");

		let flags = D3DCOMPILE_ENABLE_STRICTNESS;

		unsafe {
			let mut blob = None;

			D3DCompile(
				SHADER_TEXT.as_ptr().cast(),
				SHADER_TEXT.len(),
				None,
				None,
				None,
				S::ENTRY_POINT,
				S::TARGET,
				flags,
				0,
				&mut blob,
				None,
			)
			.map(|()| blob.unwrap_unchecked())
		}
	}
}

trait Shader: Sized {
	const ENTRY_POINT: PCSTR;
	const TARGET: PCSTR;

	fn create(device: &ID3D11Device, blob: &ID3DBlob) -> windows::core::Result<Self>;
}

impl Shader for ID3D11VertexShader {
	const ENTRY_POINT: PCSTR = s!("vs_main");
	const TARGET: PCSTR = s!("vs_5_0");

	fn create(device: &ID3D11Device, blob: &ID3DBlob) -> windows::core::Result<Self> {
		let bytecode = CompiledShaders::bytecode(blob);

		unsafe {
			let mut vertex_shader = None;
			device
				.CreateVertexShader(bytecode, None, Some(&mut vertex_shader))
				.map(|()| vertex_shader.unwrap_unchecked())
		}
	}
}

impl Shader for ID3D11PixelShader {
	const ENTRY_POINT: PCSTR = s!("ps_main");
	const TARGET: PCSTR = s!("ps_5_0");

	fn create(device: &ID3D11Device, blob: &ID3DBlob) -> windows::core::Result<Self> {
		let bytecode = CompiledShaders::bytecode(blob);

		unsafe {
			let mut pixel_shader = None;
			device
				.CreatePixelShader(bytecode, None, Some(&mut pixel_shader))
				.map(|()| pixel_shader.unwrap_unchecked())
		}
	}
}
