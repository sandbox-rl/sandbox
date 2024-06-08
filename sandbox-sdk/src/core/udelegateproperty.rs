use crate::UProperty;

#[repr(C)]
pub struct UDelegateProperty {
	_super: UProperty,
	_padding: [u8; 0x10],
}

unreal_object!(UDelegateProperty, UProperty, "Core", "DelegateProperty");
