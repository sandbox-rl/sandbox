use crate::{ueptr, UProperty};

#[repr(C)]
pub struct UArrayProperty {
	_super: UProperty,
	pub Inner: ueptr<UProperty>,
}

unreal_object!(UArrayProperty, UProperty, "Core", "ArrayProperty");
