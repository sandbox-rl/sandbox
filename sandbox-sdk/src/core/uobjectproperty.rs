use crate::{ueptr, UClass, UProperty};

#[repr(C)]
pub struct UObjectProperty {
	_super: UProperty,
	pub PropertyClass: ueptr<UClass>,
	_padding: [u8; 0x08],
}

unreal_object!(UObjectProperty, UProperty, "Core", "ObjectProperty");
