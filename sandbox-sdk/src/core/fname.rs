use std::fmt::Display;

use crate::{ueptr, FNameEntry, TArray};

#[repr(C)]
pub struct FName {
    FNameEntryId: i32,
    InstanceNumber: i32,
}

impl FName {
    pub fn Names() -> &'static TArray<Option<ueptr<FNameEntry>>> {
        unsafe { &*crate::globals::gnames().cast() }
    }
}

impl Display for FName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Some(entry) = FName::Names()[self.FNameEntryId] else {
            unreachable!();
        };

        write!(f, "{}", *entry)
    }
}
