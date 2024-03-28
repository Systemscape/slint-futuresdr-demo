/// FFT size used by FutureSDR
pub const FFT_SIZE: usize = 512;
/// Rate at which the throttle block should output samples. Usually a multiple of FFT_SIZE.
pub const PLOT_RATE: usize = FFT_SIZE * 4;

/// Rendering with plotters
pub mod rendering;

slint::include_modules!();

// If any combination of incompatible features is selected, compilation should fail
#[cfg(any(
    all(feature = "websocket_rx", feature = "futuresdr_integrated"),
    all(feature = "replay_vec", feature = "websocket_rx",),
    all(feature = "replay_vec", feature = "futuresdr_integrated")
))]
compile_error!("replay_vec, websocket_rx and futuresdr_integrated are mutually exclusive and cannot be enabled together");

#[cfg(feature = "futuresdr_integrated")]
mod futuresdr_integrated;
#[cfg(feature = "futuresdr_integrated")]
pub use futuresdr_integrated::wait_for_samples;

#[cfg(feature = "replay_vec")]
mod replay_vec;
#[cfg(feature = "replay_vec")]
pub mod vector;
#[cfg(feature = "replay_vec")]
pub use replay_vec::wait_for_samples;

#[cfg(feature = "websocket_rx")]
mod websocket_rx;

#[cfg(feature = "websocket_rx")]
pub use websocket_rx::wait_for_samples;
