use crate::{ueptr, UClass, UProperty};

#[repr(C)]
pub struct UInterfaceProperty {
	_super: UProperty,
	InterfaceClass: ueptr<UClass>,
}

unreal_object!(UInterfaceProperty, UProperty, "Core", "InterfaceProperty");
