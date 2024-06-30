use std::ffi::c_void;
use std::thread;

use windows::Win32::Foundation::{BOOL, HMODULE, TRUE};
use windows::Win32::System::LibraryLoader::DisableThreadLibraryCalls;
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;

mod devtools;
mod dx11hooks;
mod logging;
mod sandbox;
pub mod theme;
mod tracing;

use sandbox::Sandbox;

#[export_name = "DllMain"]
extern "system" fn dll_main(module: HMODULE, call_reason: u32, _reserved: *mut c_void) -> BOOL {
	if call_reason == DLL_PROCESS_ATTACH {
		let _ = unsafe { DisableThreadLibraryCalls(module) };

		let _ = thread::Builder::new()
			.name(String::from("sandbox"))
			.spawn(move || Sandbox::main(module));
	}

	TRUE
}
