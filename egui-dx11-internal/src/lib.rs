#![feature(once_cell_try)]

mod backup;
mod dx11app;
mod dx11state;
mod input;
mod mesh;
mod shader;
mod texture;
pub mod utils;

type WinResult<T> = windows::core::Result<T>;

pub use dx11app::Dx11App;
