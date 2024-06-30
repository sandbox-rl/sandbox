// use std::fmt;

// use super::ReflectionClass;
// use crate::{ueptr, UObject};

// pub struct ReflectionObject {
// 	object: ueptr<UObject>,
// 	name: String,
// 	class: ReflectionClass,
// }

// impl ReflectionObject {
// 	pub fn new(object: ueptr<UObject>) -> Self {
// 		let name = object.GetPathName();
// 		let class = ReflectionClass::new(object.Class);

// 		ReflectionObject {
// 			object,
// 			name,
// 			class,
// 		}
// 	}

// 	pub fn debug(&self) -> impl fmt::Debug + '_ {
// 		DebugReflectionObject {
// 			reflection_object: self,
// 		}
// 	}
// }

// struct DebugReflectionObject<'a> {
// 	reflection_object: &'a ReflectionObject,
// }

// impl fmt::Debug for DebugReflectionObject<'_> {
// 	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
// 		let mut d = f.debug_struct(&self.reflection_object.class.name);

// 		// TODO
// 		for prop in &self.reflection_object.class.properties {
// 			d.field(&prop.name, prop);
// 		}

// 		d.finish()
// 	}
// }
