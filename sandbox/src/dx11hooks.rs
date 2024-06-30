use std::mem;
use std::sync::Arc;

use color_eyre::eyre::Context;
use color_eyre::Result;
use egui_dx11_internal::utils::create_dummy_swapchain;
use egui_dx11_internal::Dx11App;
use retour::static_detour;
use tracing::{debug_span, error};
use windows::core::{w, Interface, HRESULT};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT;
use windows::Win32::Graphics::Dxgi::IDXGISwapChain;
use windows::Win32::UI::WindowsAndMessaging::{
	FindWindowW, GetWindowLongPtrA, GetWindowLongPtrW, GWLP_WNDPROC,
};

struct DxgiPresentHook;

impl DxgiPresentHook {
	fn new(
		present: FnPresent,
		f: impl Fn(IDXGISwapChain, u32, u32) -> HRESULT + Send + 'static,
	) -> retour::Result<Self> {
		unsafe { IDXGISwapChainPresent.initialize(present, f)?.enable()? };

		Ok(DxgiPresentHook)
	}
}

impl Drop for DxgiPresentHook {
	fn drop(&mut self) {
		if let Err(err) = unsafe { IDXGISwapChainPresent.disable() } {
			error!("Failed to unhook IDXGISwapChain::Present: {err}");
		}
	}
}

struct DxgiResizeBuffersHook;

impl DxgiResizeBuffersHook {
	fn new(
		resize_buffers: FnResizeBuffers,
		f: impl Fn(IDXGISwapChain, u32, u32, u32, DXGI_FORMAT, u32) -> HRESULT + Send + 'static,
	) -> retour::Result<Self> {
		unsafe {
			IDXGISwapChainResizeBuffers
				.initialize(resize_buffers, f)?
				.enable()?
		};

		Ok(DxgiResizeBuffersHook)
	}
}

impl Drop for DxgiResizeBuffersHook {
	fn drop(&mut self) {
		if let Err(err) = unsafe { IDXGISwapChainResizeBuffers.disable() } {
			error!("Failed to unhook IDXGISwapChain::ResizeBuffers: {err}");
		}
	}
}

struct WndProcHook;

impl WndProcHook {
	fn new(
		wndproc: FnWndProc,
		f: impl Fn(HWND, u32, WPARAM, LPARAM) -> LRESULT + Send + 'static,
	) -> retour::Result<Self> {
		unsafe { WndProc.initialize(wndproc, f)?.enable()? };

		Ok(WndProcHook)
	}
}

impl Drop for WndProcHook {
	fn drop(&mut self) {
		if let Err(err) = unsafe { WndProc.disable() } {
			error!("Failed to unhook WNDPROC: {err}");
		}
	}
}

pub struct Dx11Hooks {
	_present: DxgiPresentHook,
	_resize_buffers: DxgiResizeBuffersHook,
	_wndproc: WndProcHook,
}

impl Dx11Hooks {
	pub fn hook<T, F>(app: Dx11App<T, F>) -> Result<Self>
	where
		T: Send + 'static,
		F: FnMut(&egui::Context, &mut T) + Send,
	{
		let (present, resize_buffers) =
			Dx11Hooks::swapchain_vtable().context("Failed to get swapchain vtable")?;

		let wndproc = Dx11Hooks::wndproc();

		let app = Arc::new(app);

		let present = Dx11Hooks::hook_present(Arc::clone(&app), present)
			.context("Failed to hook IDXGISwapChain::Present")?;

		let resize_buffers = Dx11Hooks::hook_resize_buffers(Arc::clone(&app), resize_buffers)
			.context("Failed to hook IDXGISwapChain::ResizeBuffers")?;

		let wndproc =
			Dx11Hooks::hook_wndproc(Arc::clone(&app), wndproc).context("Failed to hook WNDPROC")?;

		Ok(Dx11Hooks {
			_present: present,
			_resize_buffers: resize_buffers,
			_wndproc: wndproc,
		})
	}

	fn hook_present<T, F>(
		app: Arc<Dx11App<T, F>>,
		present: FnPresent,
	) -> retour::Result<DxgiPresentHook>
	where
		T: Send + 'static,
		F: FnMut(&egui::Context, &mut T) + Send,
	{
		DxgiPresentHook::new(present, move |this, sync_interval, flags| {
			if let Err(err) = app.present(&this) {
				debug_span!("swapchain_present_hook", sync_interval, flags)
					.in_scope(|| error!("Failed on call to IDXGISwapChain::Present: {err}"));
			}

			unsafe { IDXGISwapChainPresent.call(this, sync_interval, flags) }
		})
	}

	fn hook_resize_buffers<T, F>(
		app: Arc<Dx11App<T, F>>,
		resize_buffers: FnResizeBuffers,
	) -> retour::Result<DxgiResizeBuffersHook>
	where
		T: Send + 'static,
		F: FnMut(&egui::Context, &mut T) + Send,
	{
		DxgiResizeBuffersHook::new(
			resize_buffers,
			move |this, buffer_count, width, height, new_format, flags| {
				debug_span!(
					"swapchain_resize_buffers_hook",
					buffer_count,
					width,
					height,
					?new_format,
					flags
				)
				.in_scope(|| {
					let res = app.resize_buffers(&this.clone(), move || unsafe {
						IDXGISwapChainResizeBuffers.call(
							this,
							buffer_count,
							width,
							height,
							new_format,
							flags,
						)
					});

					match res {
						Ok(res) => res,
						Err(err) => {
							error!("Failed on call to SwapChain::ResizeBuffers: {err}");
							HRESULT(0)
						}
					}
				})
			},
		)
	}

	fn hook_wndproc<T, F>(
		app: Arc<Dx11App<T, F>>,
		wndproc: FnWndProc,
	) -> retour::Result<WndProcHook>
	where
		T: Send + 'static,
		F: FnMut(&egui::Context, &mut T) + Send,
	{
		WndProcHook::new(wndproc, move |hwnd, umsg, wparam, lparam| {
			if app.wnd_proc(umsg, wparam, lparam) {
				// Capture input
				LRESULT(1)
			} else {
				unsafe { WndProc.call(hwnd, umsg, wparam, lparam) }
			}
		})
	}

	fn swapchain_vtable() -> Result<(FnPresent, FnResizeBuffers)> {
		let swapchain = create_dummy_swapchain().context("Failed to create dummy swapchain")?;
		let sc_vtable = swapchain.vtable();

		let present = sc_vtable.Present;
		let present = unsafe { mem::transmute(present) };

		let resize_buffers = sc_vtable.ResizeBuffers;
		let resize_buffers = unsafe { mem::transmute(resize_buffers) };

		Ok((present, resize_buffers))
	}

	fn wndproc() -> FnWndProc {
		let window = unsafe { FindWindowW(None, w!("Rocket League (64-bit, DX11, Cooked)")) };

		let wnd_proc_w = unsafe { GetWindowLongPtrW(window, GWLP_WNDPROC) };
		let wnd_proc_a = unsafe { GetWindowLongPtrA(window, GWLP_WNDPROC) };

		// make sure we dont get magic cookie
		// GetWindowLongPtrW should always work for vanilla RocketLeague,
		// but Bakkesmod overwrites this and we get a magic cookie
		// We get the "real" WNDPROC by calling GetWindowLongPtrA
		let wnd_proc = isize::max(wnd_proc_w, wnd_proc_a);

		let wndproc = unsafe { mem::transmute(wnd_proc) };

		wndproc
	}
}

type FnPresent = unsafe extern "system" fn(IDXGISwapChain, u32, u32) -> HRESULT;
type FnResizeBuffers =
	unsafe extern "system" fn(IDXGISwapChain, u32, u32, u32, DXGI_FORMAT, u32) -> HRESULT;
type FnWndProc = unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT;

static_detour! {
	static IDXGISwapChainPresent: unsafe extern "system" fn(IDXGISwapChain, u32, u32) -> HRESULT;
	static IDXGISwapChainResizeBuffers: unsafe extern "system" fn(IDXGISwapChain, u32, u32, u32, DXGI_FORMAT, u32) -> HRESULT;
	static WndProc: unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT;
}
