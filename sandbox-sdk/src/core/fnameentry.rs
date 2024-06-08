use std::fmt::Display;

use widestring::{WideCStr, WideChar};

#[repr(C)]
pub struct FNameEntry {
	// Flags: EObjectFlags,
	// Index: i32,
	// HashNext: *mut FNameEntry,
	_padding: [u8; 0x18],
	pub Name: [WideChar; 0x100],
}

impl FNameEntry {
	fn as_cstr(&self) -> &WideCStr {
		WideCStr::from_slice_truncate(&self.Name).expect("Missing null terminator")
	}
}

impl Display for FNameEntry {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_cstr().display())
	}
}
