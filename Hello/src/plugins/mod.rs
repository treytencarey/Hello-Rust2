//! Grouped plugin sets for the Hello application
//! Use these instead of manually adding individual plugins

mod core;

#[cfg(feature = "physics")]
mod physics;

#[cfg(feature = "networking")]
mod networking;

#[cfg(feature = "tiled")]
mod tiled;

#[cfg(feature = "ufbx")]
mod ufbx;

pub use core::*;

#[cfg(feature = "physics")]
pub use physics::*;

#[cfg(feature = "networking")]
pub use networking::*;

#[cfg(feature = "tiled")]
pub use tiled::*;

#[cfg(feature = "ufbx")]
pub use ufbx::*;

