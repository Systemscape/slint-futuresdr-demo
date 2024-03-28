use futuresdr::{
    anyhow::Result, blocks::Apply, blocks::Fft, blocks::FftDirection, blocks::SignalSourceBuilder,
    blocks::Throttle, futures::channel::mpsc::channel, futures_lite::StreamExt, macros::connect,
    num_complex::Complex32, runtime::buffer::slab::Slab, runtime::Flowgraph, runtime::Pmt,
    runtime::Runtime,
};
use log::{debug, info};
use slint::Weak;

use crate::{rendering::render_plot, MainWindow, FFT_SIZE, PLOT_RATE};

mod channel_sink;
use channel_sink::ChannelSink;

mod additive_noise;
use additive_noise::AdditiveNoise;

pub async fn wait_for_samples(window_weak: Weak<MainWindow>) -> Result<()> {
    // Store the noise value locally, so we only send a change message to the FG if it really changed
    let mut noise_val = get_noise(&window_weak.clone().unwrap());

    let mut fg = Flowgraph::new();

    // Set it to some arbitrary frequency
    let src =
        SignalSourceBuilder::<Complex32>::sin(PLOT_RATE as f32 / 4.0, PLOT_RATE as f32).build();

    // Additive Noise
    let noise = AdditiveNoise::new(noise_val);

    // Store the `gain` port ID for later use
    let gain_message_id = noise
        .message_input_name_to_id("gain")
        .expect("No gain message id found!");

    // FFT Block
    let fft = Fft::with_options(FFT_SIZE, FftDirection::Forward, false, None);

    // Throttle sample rate to allow the GUI to render the plot in the meantime
    let throttle = Throttle::<Complex32>::new(PLOT_RATE as f64);

    // We don't want to plot complex values, so take the absolute value
    let mag = Apply::new(|x: &Complex32| x.norm());

    // Create channel for the channel sink
    let (set_samples, mut samples) = channel::<Box<[f32; FFT_SIZE]>>(10);
    let snk = ChannelSink::new(set_samples);

    // Connect with custom buffers to get output in real-time.
    // Use 8*FFT_SIZE for Complex32 (2x 4 bytes) and 4*FFT_SIZE for f32 (1x 4 bytes)
    connect!(fg, src  > noise  [Slab::with_config(8 * FFT_SIZE, 2, 0)] fft [Slab::with_config(8 * FFT_SIZE, 2, 0)] throttle [Slab::with_config(4 * FFT_SIZE, 2, 0)] mag [Slab::with_config(4 * FFT_SIZE, 2, 0)] snk);

    info!("Start FG");
    let rt = Runtime::new();
    let (_, mut fg) = rt.start(fg).await;

    // Processing loop for samples in the channel
    while let Some(samples) = samples.next().await {
        // Only do the (expensive) rendering operation when plotting is enabled
        if window_weak.clone().unwrap().get_plot_enable() {
            debug!("updating");

            let window = window_weak.clone();
            slint::invoke_from_event_loop(move || {
                let app = window.unwrap();
                app.set_plot_frame(render_plot(samples.as_ref(), &app));
            })
            .expect("Start rendering");

            // Obtain the noise level from the UI
            let noise_val_new = get_noise(&window_weak.clone().unwrap());
            // Update only when it has changed to avoid potentially expensive message call to FG
            if noise_val_new != noise_val {
                noise_val = noise_val_new;
                info!("Setting noise to {noise_val}");
                fg.call(noise, gain_message_id, Pmt::F32(noise_val))
                    .await
                    .unwrap()
            }
        } else {
            // If the plot is no longer enabled we break the loop so the FT can be terminated
            break;
        }
    }

    info!("Terminate FG");
    fg.terminate_and_wait().await.expect("Terminate Flowgraph");
    info!("FG terminated");
    Ok(())
}

/// Noise SpinBox on the GUI including conversion and scaling
fn get_noise(window: &MainWindow) -> f32 {
    window.get_noise() as f32 / 10.0
}
