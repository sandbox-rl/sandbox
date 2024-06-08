use crate::{ueptr, UField};

#[repr(C)]
pub struct UStruct {
	_super: UField,
	_padding0: [u8; 0x10],
	pub SuperStruct: Option<ueptr<UStruct>>,
	pub Childern: Option<ueptr<UField>>,
	pub PropertySize: u32,
	_padding1: [u8; 0x9c],
}

unreal_object!(UStruct, UField, "Core", "Struct");
