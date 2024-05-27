use crate::UObjectProperty;

#[repr(C)]
pub struct UComponentProperty {
    _super: UObjectProperty,
}

unreal_object!(
    UComponentProperty,
    UObjectProperty,
    "Core",
    "ComponentProperty"
);
