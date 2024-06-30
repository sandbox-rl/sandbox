use std::collections::HashMap;
use std::iter;
use std::sync::LazyLock;

use crate::{ueptr, UFunction, UObject, UProperty, UScriptStruct, UState};

#[repr(C)]
pub struct UClass {
	_super: UState,
	_padding: [u8; 0x228],
}

unreal_object!(UClass, UState, "Core", "Class");

impl UClass {
	pub fn FindClass(FullName: &str) -> Option<ueptr<UClass>> {
		static CLASSES: LazyLock<HashMap<String, i32>> = LazyLock::new(|| {
			UObject::GObjObjects()
				.iter()
				.flatten()
				.filter(|obj| obj.GetFullName().starts_with("Class"))
				.map(|func| (func.GetFullName(), func.ObjectInternalInteger))
				.collect()
		});

		let class = CLASSES
			.get(FullName)
			.map(|&index| UObject::GObjObjects()[index])??;

		Some(class.ptr_cast())
	}

	pub fn iter_superclass(&self) -> impl Iterator<Item = ueptr<UClass>> {
		iter::successors(Some(ueptr::from(self)), |class| {
			class.SuperStruct.map(ueptr::ptr_cast)
		})
	}

	pub fn iter_properties(&self) -> impl Iterator<Item = ueptr<UProperty>> {
		// let superclass = self.SuperStruct.map(ueptr::ptr_cast::<UClass>);

		self.Childern.into_iter().flat_map(|children| {
			children
				.iter_next()
				.filter_map(|next| next.Cast::<UProperty>().map(ueptr::from))
				.filter(move |prop| {
					prop.ElementSize > 0
					// && superclass.is_none()
					// || superclass.is_some_and(|sc| prop.Offset as u32 >= sc.PropertySize)
				})
		})
	}

	pub fn iter_structs(&self) -> impl Iterator<Item = ueptr<UScriptStruct>> {
		self.Childern.into_iter().flat_map(|children| {
			children
				.iter_next()
				.filter_map(|next| next.Cast::<UScriptStruct>().map(ueptr::from))
		})
	}

	pub fn iter_functions(&self) -> impl Iterator<Item = ueptr<UFunction>> {
		self.Childern.into_iter().flat_map(|children| {
			children
				.iter_next()
				.filter_map(|next| next.Cast::<UFunction>().map(ueptr::from))
		})
	}
}
