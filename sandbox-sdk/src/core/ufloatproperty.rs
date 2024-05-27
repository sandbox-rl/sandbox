use crate::UProperty;

#[repr(C)]
pub struct UFloatProperty {
    _super: UProperty,
}

unreal_object!(UFloatProperty, UProperty, "Core", "FloatProperty");
