use crate::UStruct;

#[repr(C)]
pub struct UScriptStruct {
    _super: UStruct,
    _padding: [u8; 0x28],
}

unreal_object!(UScriptStruct, UStruct, "Core", "ScriptStruct");
