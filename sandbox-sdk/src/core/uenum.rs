use crate::{FName, TArray, UField};

#[repr(C)]
pub struct UEnum {
    _super: UField,
    pub Names: TArray<FName>,
}

unreal_object!(UEnum, UField, "Core", "Enum");
