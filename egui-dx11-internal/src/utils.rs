use std::mem;

use windows::core::{w, Interface};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, TRUE, WPARAM};
use windows::Win32::Graphics::Direct3D::{
	D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL_11_0, D3D_FEATURE_LEVEL_11_1,
};
use windows::Win32::Graphics::Direct3D11::{
	D3D11CreateDeviceAndSwapChain, D3D11_CREATE_DEVICE_FLAG, D3D11_SDK_VERSION,
};
use windows::Win32::Graphics::Dxgi::Common::{
	DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_MODE_DESC, DXGI_MODE_SCALING_UNSPECIFIED,
	DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED, DXGI_RATIONAL, DXGI_SAMPLE_DESC,
};
use windows::Win32::Graphics::Dxgi::{
	IDXGISwapChain, IDXGISwapChain_Vtbl, DXGI_SWAP_CHAIN_DESC,
	DXGI_SWAP_CHAIN_FLAG_ALLOW_MODE_SWITCH, DXGI_SWAP_EFFECT_DISCARD,
	DXGI_USAGE_RENDER_TARGET_OUTPUT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
	CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassExW, UnregisterClassW, CS_HREDRAW,
	CS_VREDRAW, WNDCLASSEXW, WS_EX_RIGHTSCROLLBAR, WS_OVERLAPPEDWINDOW,
};

use crate::WinResult;

pub fn create_dummy_swapchain() -> WinResult<IDXGISwapChain> {
	unsafe extern "system" fn wndproc(
		hwnd: HWND,
		msg: u32,
		wparam: WPARAM,
		lparam: LPARAM,
	) -> LRESULT {
		DefWindowProcW(hwnd, msg, wparam, lparam)
	}

	let windowclass = WNDCLASSEXW {
		cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
		style: CS_HREDRAW | CS_VREDRAW,
		lpfnWndProc: Some(wndproc),
		hInstance: unsafe { GetModuleHandleW(None).unwrap().into() },
		lpszClassName: w!("dummy"),
		..Default::default()
	};

	unsafe { RegisterClassExW(&windowclass) };

	let window = unsafe {
		CreateWindowExW(
			WS_EX_RIGHTSCROLLBAR,
			windowclass.lpszClassName,
			w!("dummy window"),
			WS_OVERLAPPEDWINDOW,
			0,
			0,
			100,
			100,
			None,
			None,
			windowclass.hInstance,
			None,
		)
	};

	let desc = DXGI_SWAP_CHAIN_DESC {
		BufferDesc: DXGI_MODE_DESC {
			Height: 100,
			Width: 100,
			RefreshRate: DXGI_RATIONAL {
				Numerator: 60,
				Denominator: 1,
			},
			Format: DXGI_FORMAT_R8G8B8A8_UNORM,
			ScanlineOrdering: DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED,
			Scaling: DXGI_MODE_SCALING_UNSPECIFIED,
		},
		SampleDesc: DXGI_SAMPLE_DESC {
			Count: 1,
			Quality: 0,
		},
		BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
		BufferCount: 1,
		OutputWindow: window,
		Windowed: TRUE,
		SwapEffect: DXGI_SWAP_EFFECT_DISCARD,
		Flags: DXGI_SWAP_CHAIN_FLAG_ALLOW_MODE_SWITCH.0 as u32,
	};

	let feature_levels = [D3D_FEATURE_LEVEL_11_0, D3D_FEATURE_LEVEL_11_1];
	let mut swapchain = None;
	let mut device = None;
	let mut context = None;

	unsafe {
		D3D11CreateDeviceAndSwapChain(
			None,
			D3D_DRIVER_TYPE_HARDWARE,
			None,
			D3D11_CREATE_DEVICE_FLAG(0),
			Some(&feature_levels),
			D3D11_SDK_VERSION,
			Some(&desc),
			Some(&mut swapchain),
			Some(&mut device),
			None,
			Some(&mut context),
		)?
	};

	let swapchain = swapchain.unwrap();

	let _ = unsafe { DestroyWindow(window) };
	let _ = unsafe { UnregisterClassW(windowclass.lpszClassName, windowclass.hInstance) };

	Ok(swapchain)
}

pub fn swapchain_vtable() -> WinResult<&'static IDXGISwapChain_Vtbl> {
	let swapchain = create_dummy_swapchain()?;
	let vtable: *const _ = swapchain.vtable();
	Ok(unsafe { &*vtable })
}
