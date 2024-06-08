use crate::UProperty;

#[repr(C)]
pub struct UStrProperty {
	_super: UProperty,
}

unreal_object!(UStrProperty, UProperty, "Core", "StrProperty");
