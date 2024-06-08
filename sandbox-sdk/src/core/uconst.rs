use crate::{FString, UField};

#[repr(C)]
pub struct UConst {
	_super: UField,
	pub Value: FString,
}

unreal_object!(UConst, UField, "Core", "Const");
