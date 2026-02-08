#![feature(portable_simd)]

/// Stable external API contract surface.
pub mod api;
/// Transitional legacy command surface kept for compatibility.
pub mod commands;
#[doc(hidden)]
pub mod conversions;
#[doc(hidden)]
pub mod dependency_injection;
#[doc(hidden)]
pub mod engine;
/// Transitional legacy event surface kept for compatibility.
pub mod events;
#[doc(hidden)]
pub mod registries;
/// Transitional legacy types surface kept for compatibility.
pub mod structures;
#[doc(hidden)]
pub mod traits;
#[doc(hidden)]
pub mod utils;
