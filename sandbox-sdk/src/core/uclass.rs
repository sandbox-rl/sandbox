use std::iter;
use std::ptr::NonNull;

use super::UState;
use crate::ueptr;

#[repr(C)]
pub struct UClass {
	_super: UState,
	_padding: [u8; 0x228],
}

unreal_object!(UClass, UState, "Core", "State");

impl UClass {
	pub(crate) fn iter_superclass(&self) -> impl Iterator<Item = ueptr<UClass>> {
		iter::successors(Some(ueptr(NonNull::from(self))), |class| {
			class.SuperStruct.map(ueptr::ptr_cast)
		})
	}
}
