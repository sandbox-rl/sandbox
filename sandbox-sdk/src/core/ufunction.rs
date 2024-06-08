use std::collections::HashMap;
use std::ptr::NonNull;
use std::sync::OnceLock;

use bitflags::bitflags;

use crate::{ueptr, FName, FPointer, UObject, UStruct};

bitflags! {
	pub struct EFunctionFlags: i64 {
		const None = 0x0000_0000;
		const Final = 0x0000_0001;
		const Defined = 0x0000_0002;
		const Iterator = 0x0000_0004;
		const Latent = 0x0000_0008;
		const PreOperator = 0x0000_0010;
		const Singular = 0x0000_0020;
		const Net = 0x0000_0040;
		const NetReliable = 0x0000_0080;
		const Simulated = 0x0000_0100;
		const Exec = 0x0000_0200;
		const Native = 0x0000_0400;
		const Event = 0x0000_0800;
		const Operator = 0x0000_1000;
		const Static = 0x0000_2000;
		const NoExport = 0x0000_4000;
		const OptionalParm = 0x0000_4000;
		const Const = 0x0000_8000;
		const Invariant = 0x0001_0000;
		const Public = 0x0002_0000;
		const Private = 0x0004_0000;
		const Protected = 0x0008_0000;
		const Delegate = 0x0010_0000;
		const NetServer = 0x0020_0000;
		const HasOutParms = 0x0040_0000;
		const HasDefaults =	0x0080_0000;
		const NetClient = 0x0100_0000;
		const DLLImport = 0x0200_0000;
		const K2Call = 0x0400_0000;
		const K2Override = 0x0800_0000;
		const K2Pure = 0x1000_0000;
		const EditorOnly = 0x2000_0000;
		const Lambda = 0x4000_0000;
		const NetValidate = 0x8000_0000;
		const AllFlags = 0xFFFF_FFFF;
	}
}

#[repr(C)]
pub struct UFunction {
	_super: UStruct,
	pub FunctionFlags: EFunctionFlags,
	pub iNative: u16,
	pub RepOffset: u16,
	pub FriendlyName: FName,
	pub OperatorPrecedence: u8,
	pub NumParms: u8,
	pub ParmsSize: u16,
	pub ReturnValueOffset: u32,
	_padding: [u8; 0xc],
	pub Func: FPointer,
}

unreal_object!(UFunction, UStruct, "Core", "Function");

impl UFunction {
	pub fn FindFunction(FullName: &str) -> Option<ueptr<UFunction>> {
		static FUNCTIONS: OnceLock<HashMap<String, i32>> = OnceLock::new();

		let functions = FUNCTIONS.get_or_init(|| {
			UObject::GObjObjects()
				.iter()
				.flatten()
				.filter_map(|obj| obj.Cast::<UFunction>())
				.map(|func| (func.GetFullName(), func.ObjectInternalInteger))
				.collect()
		});

		let function = functions
			.get(FullName)
			.map(|&index| UObject::GObjObjects()[index])??;

		Some(ueptr(NonNull::from(function.Cast()?)))
	}
}
