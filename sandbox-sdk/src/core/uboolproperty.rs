use crate::UProperty;

#[repr(C)]
pub struct UBoolProperty {
    _super: UProperty,
    pub BitMask: u64,
}

unreal_object!(UBoolProperty, UProperty, "Core", "BoolProperty");
