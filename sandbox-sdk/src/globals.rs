use std::ffi::c_void;
use std::iter;
use std::mem;
use std::num::ParseIntError;
use std::ptr::null_mut;
use std::slice;
use std::sync::LazyLock;

use itertools::Itertools;
use widestring::{widecstr, WideCStr};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Memory::PAGE_NOCACHE;
use windows::Win32::System::Memory::{
    VirtualQuery, MEMORY_BASIC_INFORMATION, MEM_COMMIT, MEM_PRIVATE, PAGE_GUARD, PAGE_NOACCESS,
    PAGE_READWRITE,
};
use windows::Win32::System::ProcessStatus::{K32GetModuleInformation, MODULEINFO};
use windows::Win32::System::SystemInformation::GetSystemInfo;
use windows::Win32::System::SystemInformation::SYSTEM_INFO;
use windows::Win32::System::Threading::GetCurrentProcess;

/// Converts a byte slice to a pattern string
fn bytes_to_pat(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).join(" ")
}

/// Converts a pointer to a pattern string
fn ptr_to_pat(ptr: *const c_void) -> String {
    let addr = ptr as usize;
    bytes_to_pat(&addr.to_le_bytes())
}

/// Gets all valid memory pages, removing duplicates and pages with incorrect permissions
fn pages() -> impl Iterator<Item = &'static [u8]> {
    let mut sysinfo = SYSTEM_INFO::default();
    unsafe { GetSystemInfo(&mut sysinfo) };
    let stop = sysinfo.lpMaximumApplicationAddress;

    let mut addr: *mut c_void = null_mut();

    iter::successors(Some(MEMORY_BASIC_INFORMATION::default()), move |_| {
        let mut mbi = MEMORY_BASIC_INFORMATION::default();
        if addr < stop
            && unsafe { VirtualQuery(Some(addr), &mut mbi, mem::size_of_val(&mbi)) } != 0
            && unsafe { addr.add(mbi.RegionSize) } > addr
        {
            addr = unsafe { addr.add(mbi.RegionSize) };
            Some(mbi)
        } else {
            None
        }
    })
    .filter(|info| {
        info.AllocationProtect == PAGE_READWRITE
            && info.State == MEM_COMMIT
            && info.Type == MEM_PRIVATE
            && info.RegionSize > 4096
            && !info.Protect.contains(PAGE_GUARD)
            && !info.Protect.contains(PAGE_NOACCESS)
            && !info.Protect.contains(PAGE_NOCACHE)
    })
    .map(|info| unsafe { slice::from_raw_parts(info.BaseAddress.cast(), info.RegionSize) })
}

/// Gets process memory as a slice. This only returns memory in the form `RocketLeague.exe+...`
/// so it misses some memory in pages before the entry address.
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

/// Parses a pattern in the form "?? ?? C0 DE ?? ??"
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

pub fn find_pattern(pat: &str, memory: &[u8]) -> Option<*const c_void> {
    let pat = parse_pattern(pat).expect("Failed to parse pattern");

    memory
        .windows(pat.len())
        .find(|mem| pat_matches(&pat, mem))
        .map(|s| s.as_ptr().cast())
}

/// Generates a pattern to find FNameEntries from "None\0" and "ByteProperty\0"
fn name_pattern() -> String {
    const NAMES: [&WideCStr; 2] = [widecstr!("None"), widecstr!("ByteProperty")];

    const PADDING: [&str; 0x18] = ["??"; 0x18];

    NAMES
        .iter()
        .flat_map(|name| {
            let pat = name
                .as_slice_with_nul()
                .iter()
                .flat_map(|c| c.to_le_bytes())
                .map(|b| format!("{b:02x}"))
                .collect::<Vec<String>>();

            let padding = PADDING.into_iter().map(str::to_string).collect();
            [padding, pat].concat()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Gets the address of `FName::Names`, the global `FNameEntry` store.
fn gnames_addr() -> Option<*const c_void> {
    // In order to find the address of `FName::Names`, we work backwards in 3 steps.
    //
    // First, we find the memory allocation of the names themselves.
    // `name_pattern` generates the pattern that we use for this, which uses the first
    // two FNames, None and ByteProperty. Of note is that while the size of FNameEntry
    // is 0x218, the entries themselves are packed, as shown below in (3).
    //
    // After finding the address of the start of the FNameEntries, we then generate a
    // pattern using the addresses of the first two entries, which will be the first
    // two elements of what `Data` points to. This is shown in (2). This pattern can
    // have multiple matches, and sometimes the first match is not the correct match.
    // To account for this, we iterate through each match until we find one that gives
    // us a second match.
    //
    // In order to find the final match, we iterate through each of the previous matches
    // and try and find a pattern match for that address. Once we find a match, we have
    // found the address of `Data`, and by proxy `FNames::Name`.
    //
    // (1) FNames::Name TArray data layout
    // FNames::Name {
    //     Data: *mut *mut FNameEntry,
    //     ..
    // }
    //
    // (2) Memory layout at `Data`
    // *mut FNameEntry, *mut FNameEntry, *mut FNameEntry
    //
    // (3) Memory layout of FNameEntries. Pointed to by the first pointer in (2)
    // Padding: [u8; 0x18] Name: w"None\0" Padding: [u8; 0x18] Name: w"ByteProperty\0"

    // This is the starting pattern, which uses the "None" and "ByteProperty" FNameEntries.
    let fnameentry = name_pattern();

    // First, we get all memory pages that we want to read as &[u8].
    let mut mem = pages();

    // Then we find the address that matches the pattern by searching through the pages.
    let fnameentry = mem.find_map(|mem| find_pattern(&fnameentry, mem))?;

    // We then convert the address to a pattern to search for the pointer.
    let gnames = ptr_to_pat(fnameentry);

    // We iterate through all of the pointers to find all addresses that match
    let gnames = mem.filter_map(|mem| find_pattern(&gnames, mem));

    // Finally, we find the first pattern that has a match, which will be the `Data` pointer
    // in `FName::Names`.
    gnames
        .map(ptr_to_pat)
        .find_map(|pat| find_pattern(&pat, memory()))
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
