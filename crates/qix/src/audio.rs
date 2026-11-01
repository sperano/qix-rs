//! The audio output the sound effects play through: the default device
//! with the `audio` feature, none without it.

#[cfg(feature = "audio")]
mod device;
#[cfg(not(feature = "audio"))]
mod silent;

#[cfg(feature = "audio")]
pub use device::Output;
#[cfg(not(feature = "audio"))]
pub use silent::Output;
