use super::ReflectionProperty;
use crate::{ueptr, EPropertyFlags, UFunction, UProperty};

pub struct ReflectionFunction {
	pub function: ueptr<UFunction>,
	pub name: String,
	pub params: Vec<ReflectionParam>,
	pub ret: Option<ReflectionRetVal>,
}

impl ReflectionFunction {
	pub fn new(function: ueptr<UFunction>) -> Self {
		let name = function.GetName();

		let mut params = function
			.iter_params()
			.filter_map(ReflectionParam::new)
			.collect::<Vec<_>>();

		params.sort_by_key(|param| param.prop.offset);

		let ret = function.ret_val().and_then(ReflectionRetVal::new);

		ReflectionFunction {
			function,
			name,
			params,
			ret,
		}
	}
}

pub struct ReflectionParam {
	pub prop: ReflectionProperty,
}

impl ReflectionParam {
	pub fn new(param: ueptr<UProperty>) -> Option<ReflectionParam> {
		let prop = ReflectionProperty::new(param);

		if prop.property.PropertyFlags.contains(EPropertyFlags::Parm) {
			Some(ReflectionParam { prop })
		} else {
			None
		}
	}
}

pub struct ReflectionRetVal {
	pub param: ReflectionParam,
}

impl ReflectionRetVal {
	pub fn new(param: ueptr<UProperty>) -> Option<ReflectionRetVal> {
		let param = ReflectionParam::new(param)?;

		if param
			.prop
			.property
			.PropertyFlags
			.contains(EPropertyFlags::ReturnParm)
		{
			Some(ReflectionRetVal { param })
		} else {
			None
		}
	}
}
