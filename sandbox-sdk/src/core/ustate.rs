use crate::UStruct;

#[repr(C)]
pub struct UState {
	_super: UStruct,
	_padding: [u8; 0x60],
}

unreal_object!(UState, UStruct, "Core", "State");
