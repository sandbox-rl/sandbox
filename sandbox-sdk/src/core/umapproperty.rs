use crate::{ueptr, UProperty};

#[repr(C)]
pub struct UMapProperty {
    _super: UProperty,
    pub Key: ueptr<UProperty>,
    pub Value: ueptr<UProperty>,
}

unreal_object!(UMapProperty, UProperty, "Core", "MapProperty");
