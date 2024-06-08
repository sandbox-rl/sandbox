use crate::{ueptr, UProperty, UStruct};

#[repr(C)]
pub struct UStructProperty {
	_super: UProperty,
	Struct: ueptr<UStruct>,
}

unreal_object!(UStructProperty, UProperty, "Core", "StructProperty");
