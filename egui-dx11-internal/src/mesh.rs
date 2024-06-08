use std::mem;

use egui::{Mesh, Pos2, Rect, Rgba, TextureId};
use windows::Win32::Graphics::Direct3D11::{
	ID3D11Buffer, ID3D11Device, D3D11_BIND_INDEX_BUFFER, D3D11_BIND_VERTEX_BUFFER,
	D3D11_BUFFER_DESC, D3D11_SUBRESOURCE_DATA, D3D11_USAGE_DEFAULT,
};

use crate::WinResult;

pub struct GpuMesh {
	pub indicies: Vec<u32>,
	pub verticies: Vec<GpuVertex>,
	pub clip: Rect,
	pub texture_id: TextureId,
}

impl GpuMesh {
	pub fn from_mesh((w, h): (f32, f32), mesh: Mesh, scissors: Rect) -> Option<Self> {
		if !mesh.indices.is_empty() && mesh.indices.len() % 3 == 0 {
			let verticies = mesh
				.vertices
				.into_iter()
				.map(|v| GpuVertex {
					pos: Pos2::new(
						(v.pos.x - w / 2.) / (w / 2.),
						(v.pos.y - h / 2.) / -(h / 2.),
					),
					uv: v.uv,
					color: v.color.into(),
				})
				.collect();

			Some(Self {
				indicies: mesh.indices,
				clip: scissors,
				texture_id: mesh.texture_id,
				verticies,
			})
		} else {
			None
		}
	}
}

#[repr(C)]
pub struct GpuVertex {
	pos: Pos2,
	uv: Pos2,
	color: Rgba,
}

pub fn create_vertex_buffer(device: &ID3D11Device, mesh: &GpuMesh) -> WinResult<ID3D11Buffer> {
	let desc = D3D11_BUFFER_DESC {
		ByteWidth: (mesh.verticies.len() * mem::size_of::<GpuVertex>()) as u32,
		Usage: D3D11_USAGE_DEFAULT,
		BindFlags: D3D11_BIND_VERTEX_BUFFER.0 as u32,
		..Default::default()
	};

	let init = D3D11_SUBRESOURCE_DATA {
		pSysMem: mesh.verticies.as_ptr().cast(),
		..Default::default()
	};

	let mut buffer = None;
	unsafe {
		device
			.CreateBuffer(&desc, Some(&init), Some(&mut buffer))
			.map(|()| buffer.unwrap_unchecked())
	}
}

pub fn create_index_buffer(device: &ID3D11Device, mesh: &GpuMesh) -> WinResult<ID3D11Buffer> {
	let desc = D3D11_BUFFER_DESC {
		ByteWidth: (mesh.indicies.len() * mem::size_of::<u32>()) as u32,
		Usage: D3D11_USAGE_DEFAULT,
		BindFlags: D3D11_BIND_INDEX_BUFFER.0 as u32,
		..Default::default()
	};

	let init = D3D11_SUBRESOURCE_DATA {
		pSysMem: mesh.indicies.as_ptr().cast(),
		..Default::default()
	};

	let mut buffer = None;
	unsafe {
		device
			.CreateBuffer(&desc, Some(&init), Some(&mut buffer))
			.map(|()| buffer.unwrap_unchecked())
	}
}
