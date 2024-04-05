use log::info;
use slint::ComponentHandle;
use slint_futuresdr_demo::{wait_for_samples, MainWindow};

#[cfg(feature = "record_to_file")]
compile_error!("Must run `record_to_file` bin when spcifying `record_to_file` feature");

// Call the lib function from the bin for wasm compatibility
//https://internals.rust-lang.org/t/pre-issue-feature-request-give-me-the-option-to-not-build-a-target/18852/12
pub fn main() {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(all(debug_assertions, target_arch = "wasm32"))]
    {
        console_error_panic_hook::set_once();
        _ = console_log::init_with_level(log::Level::Info);
    }
    #[cfg(all(not(debug_assertions), target_arch = "wasm32"))]
    {
        console_error_panic_hook::set_once();
        _ = console_log::init_with_level(futuresdr::log::Level::Warn);
    }
    info!("starting");

    let app = MainWindow::new().unwrap();

    let window_weak = app.as_weak();

    window_weak
        .clone()
        .unwrap()
        .on_plot_enable_toggled(move || {
            // Start the sample generation when plot is enabled.
            // The FG loop will check if the plot is still enabled and terminate the FG otherwise
            if window_weak.unwrap().get_plot_enable() {
                let window_weak = window_weak.clone();
                slint::spawn_local(async move { wait_for_samples(window_weak).await }).unwrap();
            }
            ()
        });

    app.run().unwrap();
}
