use std::{collections::HashMap, ptr::NonNull, sync::OnceLock};

use bitflags::bitflags;

use crate::{ueptr, FName, FPointer, UObject, UStruct};

bitflags! {
    pub struct EFunctionFlags: i64 {
        const None = 0x00000000;
        const Final = 0x00000001;
        const Defined = 0x00000002;
        const Iterator = 0x00000004;
        const Latent = 0x00000008;
        const PreOperator = 0x00000010;
        const Singular = 0x00000020;
        const Net = 0x00000040;
        const NetReliable = 0x00000080;
        const Simulated = 0x00000100;
        const Exec = 0x00000200;
        const Native = 0x00000400;
        const Event = 0x00000800;
        const Operator = 0x00001000;
        const Static = 0x00002000;
        const NoExport = 0x00004000;
        const OptionalParm = 0x00004000;
        const Const = 0x00008000;
        const Invariant = 0x00010000;
        const Public = 0x00020000;
        const Private = 0x00040000;
        const Protected = 0x00080000;
        const Delegate = 0x00100000;
        const NetServer = 0x00200000;
        const HasOutParms = 0x00400000;
        const HasDefaults =	0x00800000;
        const NetClient = 0x01000000;
        const DLLImport = 0x02000000;
        const K2Call = 0x04000000;
        const K2Override = 0x08000000;
        const K2Pure = 0x10000000;
        const EditorOnly = 0x20000000;
        const Lambda = 0x40000000;
        const NetValidate = 0x80000000;
        const AllFlags = 0xFFFFFFFF;
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
    _padding: [u8; 0xC],
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
