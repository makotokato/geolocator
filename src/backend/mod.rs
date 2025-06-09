#[cfg(target_os = "windows")]
mod windows_runtime;

#[cfg(target_os = "windows")]
pub(crate) use windows_runtime::*;