pub mod mock;

#[cfg(target_os = "linux")]
pub mod linux;

pub use mock::MockProvider;

#[cfg(target_os = "linux")]
pub use linux::LinuxProvider;
