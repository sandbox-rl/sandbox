use std::fmt;

use super::{ReflectionProperty, ReflectionStruct};
use crate::{ueptr, UClass};

pub struct ReflectionClass {
	pub class: ueptr<UClass>,
	pub superclass: Option<Box<ReflectionClass>>,
	pub name: String,
	pub properties: Vec<ReflectionProperty>,
	pub structs: Vec<ReflectionStruct>,
}

impl ReflectionClass {
	pub fn new(class: ueptr<UClass>) -> Self {
		let name = class.GetPathName();
		let superclass = class
			.SuperStruct
			.map(ueptr::ptr_cast)
			.map(ReflectionClass::new)
			.map(Box::new);

		let mut properties = class
			.iter_properties()
			.map(ReflectionProperty::new)
			.collect::<Vec<_>>();

		properties.sort_by_key(|prop| prop.offset);

		let mut structs = class
			.iter_structs()
			.map(ReflectionStruct::new)
			.collect::<Vec<_>>();

		structs.sort_by_cached_key(|s| s.name.to_owned()); // TODO: figure out how to not clone string

		ReflectionClass {
			class,
			superclass,
			name,
			properties,
			structs,
		}
	}
}

impl fmt::Debug for ReflectionClass {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut d = f.debug_struct(&self.name);

		if let Some(sc) = &self.superclass {
			d.field(&sc.class.GetName(), &sc);
		}

		for prop in &self.properties {
			d.field(&prop.name, prop);
		}

		d.finish()
	}
}
