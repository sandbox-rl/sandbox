use crate::{ueptr, UEnum, UProperty};

#[repr(C)]
pub struct UByteProperty {
	_super: UProperty,
	pub Enum: ueptr<UEnum>,
}

unreal_object!(UByteProperty, UProperty, "Core", "ByteProperty");
