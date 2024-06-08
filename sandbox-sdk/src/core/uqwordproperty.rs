use crate::UProperty;

#[repr(C)]
pub struct UQWordProperty {
	_super: UProperty,
}

unreal_object!(UQWordProperty, UProperty, "Core", "QWordProperty");
