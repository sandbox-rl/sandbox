use crate::{ueptr, UClass, UProperty};

#[repr(C)]
pub struct UInterfaceProperty {
	_super: UProperty,
	pub InterfaceClass: ueptr<UClass>,
}

unreal_object!(UInterfaceProperty, UProperty, "Core", "InterfaceProperty");
