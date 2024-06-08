use std::ffi::c_void;
use std::num::ParseIntError;
use std::sync::LazyLock;
use std::{iter, mem, slice};

use itertools::Itertools;
use memchr::memmem;
use widestring::{widecstr, WideCStr};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Memory::{
	VirtualQuery, MEMORY_BASIC_INFORMATION, MEM_COMMIT, MEM_PRIVATE, PAGE_GUARD, PAGE_NOACCESS,
	PAGE_READWRITE,
};
use windows::Win32::System::ProcessStatus::{K32GetModuleInformation, MODULEINFO};
use windows::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};
use windows::Win32::System::Threading::GetCurrentProcess;

fn gnames_addr() -> Option<*const c_void> {
	// The basic concept for locating the global names is fairly straightforward.
	//
	// We essentially work backwards from the memory layout of the global names in
	// order to find the `TArray`.
	//
	// Consider the layout of the global names:
	//
	// TArray {
	//     Data: *mut *mut FNameEntry,
	//     ..
	// }
	//
	// The `Data` pointer points to a list of pointers, which then point to the
	// FNameEntries.
	//
	// Data might point to a location in memory that looks like the following:
	//
	// 0x001ef00 0x001ef22 ...
	//
	// These pointers then point to the FNameEntries themselves. In the case of
	// Rocket League, the first FNameEntries are packed next to one another. For
	// example:
	//
	// 0x18 bytes "None\0" 0x18 bytes "ByteProperty\0" ...
	//
	// So we work backwards to determine the address of the global names.
	//
	// We first search for None and ByteProperty, using a pattern generated by
	// `name_pattern`. We then use the address found of the first pattern to search
	// for pointers to that location. This gives us another pointer. If we find that
	// pointer in global memory, that is the `Data` field of the global names, which
	// means we found the global names.
	//
	// One issue that occurs is that most of this memory is only accessible through
	// virtual pages. This means we need to iterate through the virtual pages to
	// search for this memory. The `pages` function handles this.
	//
	// Another issue that occurs is that the second pointer occurs multiple times in
	// the virtual pages. We handle this by checking each pointer in the global
	// memory, which is accessed through the `memory` function. Once we find a
	// pointer that has a match in global memory, we have found the correct pointer
	// and the global names.

	// First, we generate the pattern we search for to find the location of the
	// "None" and "ByteProperty" FNameEntries. This starts with "None", not the
	// padding before it, so we will need to offset this by the padding later.
	let fnameentry = name_pattern();

	// We then search virtual memory by iterating through the allocated pages to
	// find the generated pattern. Once we find the pattern, we subtract the
	// FNameEntry padding bytes to get the start of the FNameEntries.
	let fnameentry = pages().find_map(|mem| find_pattern(&fnameentry, mem))?;
	let fnameentry = unsafe { fnameentry.sub(FNAMEENTRY_PADDING) };

	// We then search for pointers to the address we found previously. In order to
	// search for the address, we convert the pointer into its constitutent bytes.
	// In order to optimize the page search, we create a finder using the `memchr`
	// crate.
	let fnameentry = (fnameentry as usize).to_le_bytes();
	let fnameentry = memmem::Finder::new(&fnameentry);

	// Cache the global memory so that we don't have to read it every time.
	let memory = memory();

	// This step takes the largest amount of time. There are multiple instances of
	// the address we are searching for, but only one of them will be found in the
	// global memory, which will be the address of the global names.
	//
	// We first create an iterator that searches through pages and returns the
	// address of any pointers if they are found. We search through pages using the
	// `memmem::Finder` to optimize this process.
	//
	// Next we search global memory for the bytes of the pointer we just found. Once
	// we find a match, we have found the global names `TArray`.
	pages()
		.filter_map(|mem| {
			fnameentry
				.find(mem)
				.map(|offset| unsafe { mem.as_ptr().add(offset) })
		})
		.find_map(|ptr| {
			let pat = (ptr as usize).to_le_bytes();
			memmem::find(memory, &pat).map(|offset| unsafe { memory.as_ptr().add(offset).cast() })
		})
}

const FNAMEENTRY_PADDING: usize = 0x18;

fn name_pattern() -> String {
	const NAMES: [&WideCStr; 2] = [widecstr!("None"), widecstr!("ByteProperty")];

	const PADDING: [&str; FNAMEENTRY_PADDING] = ["??"; FNAMEENTRY_PADDING];

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

fn find_pattern(pat: &str, memory: &[u8]) -> Option<*const c_void> {
	let pat = parse_pattern(pat).expect("Failed to parse pattern");

	memory
		.windows(pat.len())
		.find(|&mem| pat_matches(&pat, mem))
		.map(|s| s.as_ptr().cast())
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
