use crate::UProperty;

#[repr(C)]
pub struct UIntProperty {
	_super: UProperty,
}

unreal_object!(UIntProperty, UProperty, "Core", "IntProperty");
