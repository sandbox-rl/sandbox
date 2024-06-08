use super::UObject;
use crate::ueptr;

#[repr(C)]
pub struct UField {
	_super: UObject,
	pub Next: Option<ueptr<UField>>,
	_padding: [u8; 0x8],
}

unreal_object!(UField, UObject, "Core", "Field");
