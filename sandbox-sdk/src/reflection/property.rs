use std::fmt;

use crate::{
	ueptr, UArrayProperty, UBoolProperty, UByteProperty, UClassProperty, UDelegateProperty,
	UFloatProperty, UIntProperty, UInterfaceProperty, UMapProperty, UNameProperty, UObjectProperty,
	UProperty, UQWordProperty, UStrProperty, UStructProperty,
};

pub struct ReflectionProperty {
	pub property: ueptr<UProperty>,
	pub name: String,
	pub uetype: PropertyType,
	pub offset: usize,
}

impl ReflectionProperty {
	pub fn new(property: ueptr<UProperty>) -> Self {
		let offset = property.Offset as usize;
		let name = property.GetName();
		let uetype = ReflectionProperty::uetype(&property);

		ReflectionProperty {
			property,
			name,
			uetype,
			offset,
		}
	}

	fn uetype(property: &UProperty) -> PropertyType {
		// Native properties
		if property.IsA::<UBoolProperty>() {
			PropertyType::Native(NativeProperty::Bool)
		} else if property.IsA::<UByteProperty>() {
			PropertyType::Native(NativeProperty::U8)
		} else if property.IsA::<UIntProperty>() {
			PropertyType::Native(NativeProperty::I32)
		} else if property.IsA::<UFloatProperty>() {
			PropertyType::Native(NativeProperty::F32)
		} else if property.IsA::<UQWordProperty>() {
			PropertyType::Native(NativeProperty::U64)
		}
		// Struct properties
		else if property.IsA::<UNameProperty>() {
			PropertyType::Struct(StructProperty::FName)
		} else if property.IsA::<UStrProperty>() {
			PropertyType::Struct(StructProperty::FString)
		} else if let Some(struct_prop) = property.Cast::<UStructProperty>() {
			let name = struct_prop.Struct.GetName();
			PropertyType::Struct(StructProperty::FStruct(name))
		} else if property.IsA::<UDelegateProperty>() {
			PropertyType::Struct(StructProperty::FScriptDelegate)
		}
		// Template properties
		else if let Some(array_prop) = property.Cast::<UArrayProperty>() {
			let inner = ReflectionProperty::new(array_prop.Inner);
			PropertyType::Template(TemplateProperty::TArray(Box::new(inner.uetype)))
		} else if let Some(map_prop) = property.Cast::<UMapProperty>() {
			let key = ReflectionProperty::new(map_prop.Key);
			let val = ReflectionProperty::new(map_prop.Value);

			PropertyType::Template(TemplateProperty::TMap {
				key: Box::new(key.uetype),
				val: Box::new(val.uetype),
			})
		}
		// Pointer properties
		else if let Some(object_prop) = property.Cast::<UObjectProperty>() {
			let name = object_prop.PropertyClass.GetName();

			PropertyType::Pointer(PointerProperty::UObject(name))
		} else if let Some(class_prop) = property.Cast::<UClassProperty>() {
			let name = class_prop.MetaClass.GetName();

			PropertyType::Pointer(PointerProperty::UClass(name))
		} else if let Some(interface_prop) = property.Cast::<UInterfaceProperty>() {
			let name = interface_prop.InterfaceClass.GetName();

			PropertyType::Pointer(PointerProperty::UInterface(name))
		}
		// Nothing matched, we dont know what this is
		else {
			PropertyType::Unknown
		}
	}
}

impl fmt::Debug for ReflectionProperty {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{type} ({offset})",
			type = self.uetype.type_name(),
			offset = self.offset
		)
	}
}

#[derive(Debug)]
pub enum PropertyType {
	Native(NativeProperty),
	Struct(StructProperty),
	Pointer(PointerProperty),
	Template(TemplateProperty),
	Unknown,
}

impl PropertyType {
	pub fn type_name(&self) -> String {
		match self {
			PropertyType::Native(NativeProperty::U8) => String::from("u8"),
			PropertyType::Native(NativeProperty::I32) => String::from("i32"),
			PropertyType::Native(NativeProperty::U64) => String::from("u64"),
			PropertyType::Native(NativeProperty::F32) => String::from("f32"),
			PropertyType::Native(NativeProperty::Bool) => String::from("bool"),
			PropertyType::Struct(StructProperty::FName) => String::from("FName"),
			PropertyType::Struct(StructProperty::FString) => String::from("FString"),
			PropertyType::Struct(StructProperty::FScriptDelegate) => {
				String::from("FScriptDelegate")
			}
			PropertyType::Struct(StructProperty::FStruct(name)) => name.clone(),
			PropertyType::Template(TemplateProperty::TArray(inner)) => {
				format!("TArray<{}>", PropertyType::type_name(inner))
			}
			PropertyType::Template(TemplateProperty::TMap { key, val }) => {
				format!(
					"TMap<{}, {}>",
					PropertyType::type_name(key),
					PropertyType::type_name(val)
				)
			}
			PropertyType::Pointer(PointerProperty::UObject(name))
			| PropertyType::Pointer(PointerProperty::UClass(name))
			| PropertyType::Pointer(PointerProperty::UInterface(name)) => format!("*mut {name}"),
			PropertyType::Unknown => String::from("Unknown"),
		}
	}
}

#[derive(Debug)]
pub enum NativeProperty {
	U8,
	I32,
	U64,
	F32,
	Bool,
}

#[derive(Debug)]
pub enum StructProperty {
	FName,
	FString,
	FScriptDelegate,
	FStruct(String),
}

#[derive(Debug)]
pub enum PointerProperty {
	UObject(String),
	UClass(String),
	UInterface(String),
}

#[derive(Debug)]
pub enum TemplateProperty {
	TArray(Box<PropertyType>),
	TMap {
		key: Box<PropertyType>,
		val: Box<PropertyType>,
	},
}
