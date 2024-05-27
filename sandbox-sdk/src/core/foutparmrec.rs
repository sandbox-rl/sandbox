use crate::{ueptr, UProperty};

#[repr(C)]
pub struct FOutParmRec {
    Property: Option<ueptr<UProperty>>,
    PropAddr: Option<ueptr<u8>>,
    NextOutParm: Option<ueptr<FOutParmRec>>,
}
