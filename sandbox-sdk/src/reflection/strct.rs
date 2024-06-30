use std::fmt;

use super::ReflectionProperty;
use crate::{ueptr, UProperty, UScriptStruct};

pub struct ReflectionStruct {
	pub strct: ueptr<UScriptStruct>,
	pub name: String,
	pub properties: Vec<ReflectionProperty>,
}

impl ReflectionStruct {
	pub fn new(strct: ueptr<UScriptStruct>) -> Self {
		let name = strct.GetName();
		let mut properties = strct
			.Childern
			.iter()
			.flat_map(|c| c.iter_next())
			.filter_map(|prop| prop.IsA::<UProperty>().then(|| prop.ptr_cast()))
			.map(ReflectionProperty::new)
			.collect::<Vec<_>>();

		properties.sort_by_key(|prop| prop.offset);

		ReflectionStruct {
			strct,
			name,
			properties,
		}
	}
}

impl fmt::Debug for ReflectionStruct {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut d = f.debug_struct(&self.name);

		for prop in &self.properties {
			d.field(&prop.name, prop);
		}

		d.finish()
	}
}
