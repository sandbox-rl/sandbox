use std::iter;

use super::UObject;
use crate::ueptr;

#[repr(C)]
pub struct UField {
	_super: UObject,
	pub Next: Option<ueptr<UField>>,
	_padding: [u8; 0x8],
}

unreal_object!(UField, UObject, "Core", "Field");

impl UField {
	pub(crate) fn iter_next(&self) -> impl Iterator<Item = ueptr<UField>> {
		iter::successors(Some(ueptr::from(self)), |next| next.Next)
	}
}
