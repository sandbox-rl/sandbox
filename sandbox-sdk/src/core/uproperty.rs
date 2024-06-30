use bitflags::bitflags;

use crate::UField;

bitflags! {
	pub struct EPropertyFlags: u64 {
		const Edit = 0x0000_0000_0000_0001;
		const Const = 0x0000_0000_0000_0002;
		const Input = 0x0000_0000_0000_0004;
		const ExportObject = 0x0000_0000_0000_0008;
		const OptionalParm = 0x0000_0000_0000_0010;
		const Net = 0x0000_0000_0000_0020;
		const EditConstArray = 0x0000_0000_0000_0040;
		const Parm = 0x0000_0000_0000_0080;
		const OutParm = 0x0000_0000_0000_0100;
		const SkipParm = 0x0000_0000_0000_0200;
		const ReturnParm = 0x0000_0000_0000_0400;
		const CoerceParm = 0x0000_0000_0000_0800;
		const Native = 0x0000_0000_0000_1000;
		const Transient = 0x0000_0000_0000_2000;
		const Config = 0x0000_0000_0000_4000;
		const Localized = 0x0000_0000_0000_8000;
		const Travel = 0x0000_0000_0001_0000;
		const EditConst = 0x0000_0000_0002_0000;
		const GlobalConfig = 0x0000_0000_0004_0000;
		const Component = 0x0000_0000_0008_0000;
		const NeedCtorLink = 0x0000_0000_0040_0000;
		const NoExport = 0x0000_0000_0080_0000;
		const NoClear = 0x0000_0000_0200_0000;
		const EditInline = 0x0000_0000_0400_0000;
		const EdFindable = 0x0000_0000_0800_0000;
		const EditInlineUse = 0x0000_0000_1000_0000;
		const Deprecated = 0x0000_0000_2000_0000;
		const EditInlineNotify = 0x0000_0000_4000_0000;
		const RepNotify = 0x0000_0001_0000_0000;
		const Interp = 0x0000_0002_0000_0000;
		const NonTransactional = 0x0000_0004_0000_0000;
		const EditorOnly = 0x0000_0008_0000_0000;
		const NoDestructor = 0x0000_0010_0000_0000;
		const AutoWeak = 0x0000_0040_0000_0000;
		const ContainsInstancedReference = 0x0000_0080_0000_0000;
		const AssetRegistrySearchable = 0x0000_0100_0000_0000;
		const SimpleDisplay = 0x0000_0200_0000_0000;
		const AdvancedDisplay = 0x0000_0400_0000_0000;
		const Protected = 0x0000_0800_0000_0000;
		const BlueprintCallable = 0x0000_1000_0000_0000;
		const BlueprintAuthorityOnly = 0x0000_2000_0000_0000;
		const TextExportTransient = 0x0000_4000_0000_0000;
		const NonPIEDuplicateTransient = 0x0000_8000_0000_0000;
		const ExposeOnSpawn = 0x0001_0000_0000_0000;
		const PersistentInstance = 0x0002_0000_0000_0000;
		const UObjectWrapper = 0x0004_0000_0000_0000;
		const HasGetValueTypeHash = 0x0008_0000_0000_0000;
		const NativeAccessSpecifierPublic = 0x0010_0000_0000_0000;
		const NativeAccessSpecifierProtected = 0x0020_0000_0000_0000;
		const NativeAccessSpecifierPrivate = 0x0040_0000_0000_0000;
		const SkipSerialization = 0x0080_0000_0000_0000;
	}
}

#[repr(C)]
pub struct UProperty {
	_super: UField,
	pub ArrayDim: u32,
	pub ElementSize: u32,
	pub PropertyFlags: EPropertyFlags,
	_padding_0: [u8; 0x10],
	pub PropertySize: u32,
	_padding_1: [u8; 0x4],
	pub Offset: u32,
	_padding_2: [u8; 0x2c],
}

unreal_object!(UProperty, UField, "Core", "Property");
