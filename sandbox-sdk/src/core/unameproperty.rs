use crate::UProperty;

#[repr(C)]
pub struct UNameProperty {
	_super: UProperty,
}

unreal_object!(UNameProperty, UProperty, "Core", "NameProperty");
