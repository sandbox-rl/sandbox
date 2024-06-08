use crate::{ueptr, FOutParmRec, FOutputDevice, UObject, UStruct};

#[repr(C)]
pub struct FFrame {
	_super: FOutputDevice,
	Node: Option<ueptr<UStruct>>,
	Object: Option<ueptr<UObject>>,
	Code: Option<ueptr<u8>>,
	Locals: Option<ueptr<u8>>,
	PreviousFrame: Option<ueptr<FFrame>>,
	OutParms: Option<ueptr<FOutParmRec>>,
}
