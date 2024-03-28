use slint_futuresdr_demo::FFT_SIZE;

use futuresdr::{
    anyhow::Result, blocks::Apply, blocks::Fft, blocks::FftDirection, blocks::SignalSourceBuilder,
    blocks::Throttle, blocks::WebsocketSinkBuilder, blocks::WebsocketSinkMode, macros::connect,
    num_complex::Complex32, runtime::Flowgraph, runtime::Runtime,
};

use rand::Rng;

fn main() -> Result<()> {
    let mut fg = Flowgraph::new();

    let src = SignalSourceBuilder::<Complex32>::sin(480.0, 48_000.0).build();
    let noise =
        Apply::new(|i: &Complex32| i + rand::thread_rng().gen_range(-100..100) as f32 / 30.0);
    let throttle = Throttle::<Complex32>::new(FFT_SIZE as f64 * 10.0);
    let fft = Fft::with_options(FFT_SIZE, FftDirection::Forward, true, None);
    let mag = Apply::new(|x: &Complex32| x.norm());

    let snk = WebsocketSinkBuilder::<f32>::new(9001)
        .mode(WebsocketSinkMode::FixedDropping(FFT_SIZE))
        .build();

    connect!(fg, src  > noise > fft > throttle > mag > snk);

    Runtime::new().run(fg)?;

    Ok(())
}
