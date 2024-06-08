use core::slice;
use std::collections::HashMap;
use std::mem;

use egui::{Color32, ImageData, TextureId, TexturesDelta};
use parking_lot::Mutex;
use windows::Win32::Graphics::Direct3D::D3D11_SRV_DIMENSION_TEXTURE2D;
use windows::Win32::Graphics::Direct3D11::{
	ID3D11Device, ID3D11DeviceContext, ID3D11ShaderResourceView, ID3D11Texture2D,
	D3D11_BIND_SHADER_RESOURCE, D3D11_CPU_ACCESS_WRITE, D3D11_MAP_WRITE_DISCARD,
	D3D11_SHADER_RESOURCE_VIEW_DESC, D3D11_SHADER_RESOURCE_VIEW_DESC_0, D3D11_SUBRESOURCE_DATA,
	D3D11_TEX2D_SRV, D3D11_TEXTURE2D_DESC, D3D11_USAGE_DYNAMIC,
};
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_SAMPLE_DESC};

use crate::WinResult;

pub struct ManagedTexture {
	resource: ID3D11ShaderResourceView,
	texture: ID3D11Texture2D,
	pixels: Vec<Color32>,
	width: usize,
}

#[derive(Default)]
pub struct TextureAllocator {
	allocated: Mutex<HashMap<TextureId, ManagedTexture>>,
}

impl TextureAllocator {
	pub fn process_deltas(
		&self,
		device: &ID3D11Device,
		context: &ID3D11DeviceContext,
		delta: TexturesDelta,
	) -> WinResult<()> {
		for (tex_id, delta) in delta.set {
			if delta.is_whole() {
				self.allocate_new(device, tex_id, delta.image)?;
			} else {
				self.update_partial(context, tex_id, delta.image, delta.pos.unwrap())?;
			}
		}

		for tex_id in delta.free {
			self.free(tex_id);
		}

		Ok(())
	}

	pub fn get_by_id(&self, tex_id: TextureId) -> Option<ID3D11ShaderResourceView> {
		self.allocated
			.lock()
			.get(&tex_id)
			.map(|t| t.resource.clone())
	}

	fn allocate_new(
		&self,
		device: &ID3D11Device,
		tex_id: TextureId,
		image: ImageData,
	) -> WinResult<()> {
		let tex = Self::allocate_texture(device, image)?;
		self.allocated.lock().insert(tex_id, tex);
		Ok(())
	}

	fn update_partial(
		&self,
		context: &ID3D11DeviceContext,
		tex_id: TextureId,
		image: ImageData,
		[nx, ny]: [usize; 2],
	) -> WinResult<()> {
		if let Some(old) = self.allocated.lock().get_mut(&tex_id) {
			let mut subr = Default::default();
			let subr = unsafe {
				context
					.Map(&old.texture, 0, D3D11_MAP_WRITE_DISCARD, 0, Some(&mut subr))
					.map(|()| subr.pData)?
			};

			match image {
				ImageData::Font(f) => unsafe {
					let data = slice::from_raw_parts_mut(subr as *mut Color32, old.pixels.len());
					data.as_mut_ptr()
						.copy_from_nonoverlapping(old.pixels.as_ptr(), old.pixels.len());

					let new: Vec<_> = f
						.pixels
						.iter()
						.map(|a| Color32::from_rgba_premultiplied(255, 255, 255, (a * 255.) as u8))
						.collect();

					for y in 0..f.height() {
						for x in 0..f.width() {
							let whole = (ny + y) * old.width + nx + x;
							let frac = y * f.width() + x;
							old.pixels[whole] = new[frac];
							data[whole] = new[frac];
						}
					}
				},
				_ => unreachable!(),
			}

			unsafe { context.Unmap(&old.texture, 0) };
		}

		Ok(())
	}

	fn free(&self, tex_id: TextureId) {
		self.allocated.lock().remove(&tex_id);
	}

	fn allocate_texture(device: &ID3D11Device, image: ImageData) -> WinResult<ManagedTexture> {
		let desc = D3D11_TEXTURE2D_DESC {
			Width: image.width() as _,
			Height: image.height() as _,
			MipLevels: 1,
			ArraySize: 1,
			Format: DXGI_FORMAT_R8G8B8A8_UNORM,
			SampleDesc: DXGI_SAMPLE_DESC {
				Count: 1,
				Quality: 0,
			},
			Usage: D3D11_USAGE_DYNAMIC,
			BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as _,
			CPUAccessFlags: D3D11_CPU_ACCESS_WRITE.0 as _,
			..Default::default()
		};

		let width = image.width();
		let pixels = match image {
			ImageData::Color(c) => c.pixels.clone(),
			ImageData::Font(f) => f
				.pixels
				.iter()
				.map(|a| Color32::from_rgba_premultiplied(255, 255, 255, (a * 255.) as u8))
				.collect(),
		};

		let data = D3D11_SUBRESOURCE_DATA {
			pSysMem: pixels.as_ptr().cast(),
			SysMemPitch: (width * mem::size_of::<Color32>()) as u32,
			SysMemSlicePitch: 0,
		};

		let mut texture = None;
		let texture = unsafe {
			device
				.CreateTexture2D(&desc, Some(&data), Some(&mut texture))
				.map(|()| texture.unwrap_unchecked())?
		};

		let desc = D3D11_SHADER_RESOURCE_VIEW_DESC {
			Format: DXGI_FORMAT_R8G8B8A8_UNORM,
			ViewDimension: D3D11_SRV_DIMENSION_TEXTURE2D,
			Anonymous: D3D11_SHADER_RESOURCE_VIEW_DESC_0 {
				Texture2D: D3D11_TEX2D_SRV {
					MostDetailedMip: 0,
					MipLevels: desc.MipLevels,
				},
			},
		};

		let mut resource = None;
		let resource = unsafe {
			device
				.CreateShaderResourceView(&texture, Some(&desc), Some(&mut resource))
				.map(|()| resource.unwrap_unchecked())?
		};

		Ok(ManagedTexture {
			resource,
			texture,
			pixels,
			width,
		})
	}
}
