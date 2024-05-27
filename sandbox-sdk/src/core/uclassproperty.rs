use crate::{ueptr, UClass, UObjectProperty};

#[repr(C)]
pub struct UClassProperty {
    _super: UObjectProperty,
    pub MetaClass: ueptr<UClass>,
}

unreal_object!(UClassProperty, UObjectProperty, "Core", "ClassProperty");
