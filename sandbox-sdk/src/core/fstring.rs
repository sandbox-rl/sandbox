use widestring::WideChar;

use crate::TArray;

#[repr(transparent)]
pub struct FString(TArray<WideChar>);
