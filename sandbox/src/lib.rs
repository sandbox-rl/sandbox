use std::ffi::c_void;
use std::thread;
use std::time::{Duration, Instant};

use sandbox_sdk::{FName, UObject};
use windows::Win32::Foundation::{BOOL, HMODULE, TRUE};
use windows::Win32::System::Console::{AllocConsole, FreeConsole};
use windows::Win32::System::LibraryLoader::{DisableThreadLibraryCalls, FreeLibraryAndExitThread};
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_END};

fn main(module: HMODULE) -> ! {
    let _ = unsafe { AllocConsole() };

    println!("Welcome to Sandbox");

    let start = Instant::now();
    let _ = FName::Names();
    let end = start.elapsed();

    println!("Initialized in {} seconds", end.as_secs_f64());

    let name = &*FName::Names()[0].unwrap();
    let obj = &*UObject::GObjObjects()[0].unwrap();

    println!("{name}");
    println!("{}", obj.GetFullName());

    while unsafe { GetAsyncKeyState(VK_END.0 as _) } == 0 {
        thread::sleep(Duration::from_millis(10));
    }

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
