use std::collections::HashMap;
use std::sync::LazyLock;

use crate::{ueptr, UObject, UStruct};

#[repr(C)]
pub struct UScriptStruct {
	_super: UStruct,
	_padding: [u8; 0x28],
}

unreal_object!(UScriptStruct, UStruct, "Core", "ScriptStruct");

impl UScriptStruct {
	pub fn FindStruct(FullName: &str) -> Option<ueptr<UScriptStruct>> {
		static STRUCTS: LazyLock<HashMap<String, i32>> = LazyLock::new(|| {
			UObject::GObjObjects()
				.iter()
				.flatten()
				.filter(|obj| obj.IsA::<UScriptStruct>())
				.map(|func| (func.GetFullName(), func.ObjectInternalInteger))
				.collect()
		});

		let strct = STRUCTS
			.get(FullName)
			.map(|&index| UObject::GObjObjects()[index])??;

		Some(strct.ptr_cast())
	}
}
