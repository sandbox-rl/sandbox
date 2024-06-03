use std::ffi::c_void;
use std::iter;
use std::mem;
use std::num::ParseIntError;
use std::slice;
use std::sync::LazyLock;

use itertools::Itertools;
use memchr::memmem;
use widestring::{widecstr, WideCStr};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Memory::{
    VirtualQuery, MEMORY_BASIC_INFORMATION, MEM_COMMIT, MEM_PRIVATE, PAGE_GUARD, PAGE_NOACCESS,
    PAGE_READWRITE,
};
use windows::Win32::System::ProcessStatus::{K32GetModuleInformation, MODULEINFO};
use windows::Win32::System::SystemInformation::GetSystemInfo;
use windows::Win32::System::SystemInformation::SYSTEM_INFO;
use windows::Win32::System::Threading::GetCurrentProcess;

fn pages() -> impl Iterator<Item = &'static [u8]> {
    let mut sysinfo = SYSTEM_INFO::default();
    unsafe { GetSystemInfo(&mut sysinfo) };

    let mut addr = sysinfo.lpMinimumApplicationAddress;

    iter::successors(Some(MEMORY_BASIC_INFORMATION::default()), move |pageinfo| {
        addr = unsafe { addr.add(pageinfo.RegionSize) };
        let mut pageinfo = *pageinfo;
        unsafe { VirtualQuery(Some(addr), &mut pageinfo, mem::size_of_val(&pageinfo)) };
        Some(pageinfo)
    })
    .dedup_by(|a, b| {
        a.AllocationBase == b.AllocationBase && a.AllocationProtect == b.AllocationProtect
    })
    .filter(|info| {
        info.AllocationBase == info.BaseAddress
            && info.AllocationProtect == PAGE_READWRITE
            && info.State == MEM_COMMIT
            && info.Type == MEM_PRIVATE
            && info.RegionSize > 4096
            && !info.Protect.contains(PAGE_GUARD)
            && !info.Protect.contains(PAGE_NOACCESS)
    })
    .map(|info| unsafe { slice::from_raw_parts(info.BaseAddress.cast(), info.RegionSize) })
}

fn memory() -> &'static [u8] {
    let module = unsafe { GetModuleHandleW(None).unwrap() };

    let mut mod_info = MODULEINFO::default();
    let _ = unsafe {
        K32GetModuleInformation(
            GetCurrentProcess(),
            module,
            &mut mod_info,
            mem::size_of::<MODULEINFO>() as u32,
        )
    };

    unsafe {
        slice::from_raw_parts(
            mod_info.lpBaseOfDll.cast::<u8>(),
            mod_info.SizeOfImage as usize,
        )
    }
}

fn parse_pattern(pat: &str) -> Result<Vec<Option<u8>>, ParseIntError> {
    pat.split_ascii_whitespace()
        .map(|pat| {
            (pat != "??")
                .then(|| u8::from_str_radix(pat, 16))
                .transpose()
        })
        .collect::<Result<_, _>>()
}

fn pat_matches(pat: &[Option<u8>], mem: &[u8]) -> bool {
    pat.iter()
        .zip(mem)
        .all(|(pat, &byte)| pat.is_none() || pat.is_some_and(|p| p == byte))
}

fn find_pattern(pat: &str, memory: &[u8]) -> Option<*const c_void> {
    let pat = parse_pattern(pat).expect("Failed to parse pattern");

    memory
        .windows(pat.len())
        .find(|&mem| pat_matches(&pat, mem))
        .map(|s| s.as_ptr().cast())
}

fn name_pattern() -> String {
    const NAMES: [&WideCStr; 2] = [widecstr!("None"), widecstr!("ByteProperty")];

    const PADDING: [&str; 0x18] = ["??"; 0x18];

    Itertools::intersperse_with(
        NAMES.iter().map(|name| {
            name.as_slice_with_nul()
                .iter()
                .flat_map(|c| c.to_le_bytes())
                .map(|b| format!("{b:02x}"))
                .collect::<Vec<_>>()
                .join(" ")
        }),
        || {
            PADDING
                .into_iter()
                .map(str::to_string)
                .collect::<Vec<String>>()
                .join(" ")
        },
    )
    .collect::<Vec<String>>()
    .join(" ")
}

fn gnames_addr() -> Option<*const c_void> {
    let fnameentry = name_pattern();

    let fnameentry = pages().find_map(|mem| find_pattern(&fnameentry, mem))?;
    let fnameentry = unsafe { fnameentry.sub(0x18) };

    let fnameentry = (fnameentry as usize).to_le_bytes();
    let fnameentry = memmem::Finder::new(&fnameentry);

    pages()
        .filter_map(|mem| {
            fnameentry
                .find(mem)
                .map(|offset| unsafe { mem.as_ptr().add(offset) })
        })
        .find_map(|ptr| {
            let mem = memory();
            let pat = (ptr as usize).to_le_bytes();
            memmem::find(mem, &pat).map(|offset| unsafe { mem.as_ptr().add(offset).cast() })
        })
}

pub fn gnames() -> *const c_void {
    static GNAMES: LazyLock<usize> =
        LazyLock::new(|| gnames_addr().expect("Getting GNames failed") as usize);

    *GNAMES as *mut c_void
}

pub fn gobjects() -> *const c_void {
    static GOBJECTS: LazyLock<usize> =
        LazyLock::new(|| unsafe { gnames().byte_add(0x48) as usize });

    *GOBJECTS as *mut c_void
}
