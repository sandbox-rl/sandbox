use crate::FPointer;

#[repr(C)]
pub struct FOutputDevice {
	VfTableObject: FPointer,
	AllowSuppression: bool,
	SuppressEventTag: bool,
	AutoemitLineTerminator: bool,
}
