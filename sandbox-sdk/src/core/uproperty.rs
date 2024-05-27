use bitflags::bitflags;

use crate::UField;

bitflags! {
    struct EPropertyFlags: u64 {
        const Edit = 0x0000000000000001;
        const Const = 0x0000000000000002;
        const Input = 0x0000000000000004;
        const ExportObject = 0x0000000000000008;
        const OptionalParm = 0x0000000000000010;
        const Net = 0x0000000000000020;
        const EditConstArray = 0x0000000000000040;
        const Parm = 0x0000000000000080;
        const OutParm = 0x0000000000000100;
        const SkipParm = 0x0000000000000200;
        const ReturnParm = 0x0000000000000400;
        const CoerceParm = 0x0000000000000800;
        const Native = 0x0000000000001000;
        const Transient = 0x0000000000002000;
        const Config = 0x0000000000004000;
        const Localized = 0x0000000000008000;
        const Travel = 0x0000000000010000;
        const EditConst = 0x0000000000020000;
        const GlobalConfig = 0x0000000000040000;
        const Component = 0x0000000000080000;
        const NeedCtorLink = 0x0000000000400000;
        const NoExport = 0x0000000000800000;
        const NoClear = 0x0000000002000000;
        const EditInline = 0x0000000004000000;
        const EdFindable = 0x0000000008000000;
        const EditInlineUse = 0x0000000010000000;
        const Deprecated = 0x0000000020000000;
        const EditInlineNotify = 0x0000000040000000;
        const RepNotify = 0x0000000100000000;
        const Interp = 0x0000000200000000;
        const NonTransactional = 0x0000000400000000;
        const EditorOnly = 0x0000000800000000;
        const NoDestructor = 0x0000001000000000;
        const AutoWeak = 0x0000004000000000;
        const ContainsInstancedReference = 0x0000008000000000;
        const AssetRegistrySearchable = 0x0000010000000000;
        const SimpleDisplay = 0x0000020000000000;
        const AdvancedDisplay = 0x0000040000000000;
        const Protected = 0x0000080000000000;
        const BlueprintCallable = 0x0000100000000000;
        const BlueprintAuthorityOnly = 0x0000200000000000;
        const TextExportTransient = 0x0000400000000000;
        const NonPIEDuplicateTransient = 0x0000800000000000;
        const ExposeOnSpawn = 0x0001000000000000;
        const PersistentInstance = 0x0002000000000000;
        const UObjectWrapper = 0x0004000000000000;
        const HasGetValueTypeHash = 0x0008000000000000;
        const NativeAccessSpecifierPublic = 0x0010000000000000;
        const NativeAccessSpecifierProtected = 0x0020000000000000;
        const NativeAccessSpecifierPrivate = 0x0040000000000000;
        const SkipSerialization = 0x0080000000000000;
    }
}

#[repr(C)]
pub struct UProperty {
    _super: UField,
    pub ArrayDim: i32,
    pub ElementSize: i32,
    pub PropertyFlags: EPropertyFlags,
    _padding_0: [u8; 0x10],
    pub PropertySize: u32,
    pub Offset: i32,
    _padding_1: [u8; 0x2C],
}

unreal_object!(UProperty, UField, "Core", "Property");
