use std::{iter, ptr::NonNull};

use crate::ueptr;

use super::UState;

#[repr(C)]
pub struct UClass {
    _super: UState,
    _padding: [u8; 0x228],
}

unreal_object!(UClass, UState, "Core", "State");

impl UClass {
    pub(crate) fn iter_superclass(&self) -> impl Iterator<Item = ueptr<UClass>> {
        iter::successors(Some(ueptr(NonNull::from(self))), |class| {
            class
                .SuperStruct
                .map(|super_class| super_class.ptr_cast::<UClass>())
        })
    }
}
