use std::ffi::c_void;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::{mem, thread};

use egui::Context;
use egui_dx11_internal::utils::create_dummy_swapchain;
use egui_dx11_internal::Dx11App;
use retour::static_detour;
use sandbox_sdk::{FName, UObject};
use windows::core::{w, Interface, HRESULT};
use windows::Win32::Foundation::{BOOL, HMODULE, HWND, LPARAM, LRESULT, TRUE, WPARAM};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT;
use windows::Win32::Graphics::Dxgi::IDXGISwapChain;
use windows::Win32::System::Console::{AllocConsole, FreeConsole};
use windows::Win32::System::LibraryLoader::{DisableThreadLibraryCalls, FreeLibraryAndExitThread};
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_END};
use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, GetWindowLongPtrW, GWLP_WNDPROC};

static_detour! {
	static IDXGISwapChainPresent: unsafe extern "system" fn(IDXGISwapChain, u32, u32) -> HRESULT;
	static IDXGISwapChainResizeBuffers: unsafe extern "system" fn(IDXGISwapChain, u32, u32, u32, DXGI_FORMAT, u32) -> HRESULT;
	static WndProc: unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT;
}

struct SandboxState {
	show_main: bool,
	counter: u16,
}

impl SandboxState {
	pub fn new() -> Self {
		SandboxState {
			show_main: true,
			counter: 0,
		}
	}
}

fn main(module: HMODULE) -> ! {
	let _ = unsafe { AllocConsole() };

	println!("Welcome to Sandbox");
	println!();

	let start = Instant::now();
	let _ = FName::Names();
	let end = start.elapsed();

	println!("Initialized in {} seconds", end.as_secs_f64());

	let name = FName::Names()[0].unwrap();
	let obj = UObject::GObjObjects()[0].unwrap();

	println!("{}", *name);
	println!("{}", obj.GetFullName());

	let ui = move |ctx: &Context, state: &mut SandboxState| {
		egui::Window::new("Sandbox")
			.open(&mut state.show_main)
			.show(ctx, |ui| {
				ui.label("Welcome to Sandbox");
				ui.label(format!("Initialized in {} seconds", end.as_secs_f64()));
				ui.label(name.to_string());
				ui.label(obj.GetFullName());

				if ui.button(format!("A counter: {}", state.counter)).clicked() {
					state.counter += 1;
				}
			});
	};

	let app = Dx11App::new(ui, SandboxState::new());
	let app = Arc::new(app);

	let swapchain = create_dummy_swapchain().expect("Failed to create dummy swapchain");

	let present = swapchain.vtable().Present;
	let present = unsafe { mem::transmute(present) };

	let resize_buffers = swapchain.vtable().ResizeBuffers;
	let resize_buffers = unsafe { mem::transmute(resize_buffers) };

	let window = unsafe { FindWindowW(None, w!("Rocket League (64-bit, DX11, Cooked)")) };

	let wnd_proc = unsafe { GetWindowLongPtrW(window, GWLP_WNDPROC) };
	let wnd_proc = unsafe { mem::transmute(wnd_proc) };

	unsafe {
		let app = Arc::clone(&app);

		IDXGISwapChainPresent
			.initialize(present, move |this, sync_interval, flags| {
				let _ = app.present(&this);
				IDXGISwapChainPresent.call(this, sync_interval, flags)
			})
			.expect("Failed to initialize present hook")
			.enable()
			.expect("Failed to enable present hook")
	};

	unsafe {
		let app = Arc::clone(&app);

		IDXGISwapChainResizeBuffers
			.initialize(
				resize_buffers,
				move |this, buffer_count, width, height, new_format, swap_chain_flags| {
					let res = app.resize_buffers(&this.clone(), move || {
						IDXGISwapChainResizeBuffers.call(
							this,
							buffer_count,
							width,
							height,
							new_format,
							swap_chain_flags,
						)
					});
					match res {
						Ok(res) => res,
						Err(_) => HRESULT(0),
					}
				},
			)
			.expect("Failed to initialize resize buffers hook")
			.enable()
			.expect("Failed to enable resize buffers hook");
	}

	unsafe {
		let app = Arc::clone(&app);

		WndProc
			.initialize(wnd_proc, move |hwnd, umsg, wparam, lparam| {
				if app.wnd_proc(umsg, wparam, lparam) {
					// Capture input
					LRESULT(1)
				} else {
					WndProc.call(hwnd, umsg, wparam, lparam)
				}
			})
			.expect("Failed to initialize WNDPROC hook")
			.enable()
			.expect("Failed to enable WNDPROC hook")
	};

	while unsafe { GetAsyncKeyState(VK_END.0 as _) } == 0 {
		thread::sleep(Duration::from_millis(10));
	}

	unsafe {
		IDXGISwapChainPresent
			.disable()
			.expect("Failed to disable present hook")
	};

	unsafe {
		IDXGISwapChainResizeBuffers
			.disable()
			.expect("Failed to disable resize buffers hook")
	};

	unsafe {
		WndProc.disable().expect("Failed to disable WNDPROC hook");
	}

	thread::sleep(Duration::from_millis(10));

	let _ = unsafe { FreeConsole() };
	unsafe { FreeLibraryAndExitThread(module, 0) }
}

#[export_name = "DllMain"]
extern "system" fn dll_main(module: HMODULE, call_reason: u32, _reserved: *mut c_void) -> BOOL {
	if call_reason == DLL_PROCESS_ATTACH {
		let _ = unsafe { DisableThreadLibraryCalls(module) };
		thread::spawn(move || main(module));
	}

	TRUE
}
