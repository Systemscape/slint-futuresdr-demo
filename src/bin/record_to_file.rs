use const_gen::{const_declaration, CompileConst};
use futuresdr::{
    anyhow::Result,
    blocks::Apply,
    blocks::Fft,
    blocks::FftDirection,
    blocks::SignalSourceBuilder,
    blocks::Throttle,
    blocks::{Head, VectorSink},
    macros::connect,
    num_complex::Complex32,
    runtime::Flowgraph,
    runtime::Runtime,
};
use slint_futuresdr_demo::FFT_SIZE;
use rand::Rng;

fn main() -> Result<()> {
    let mut fg = Flowgraph::new();

    let src = SignalSourceBuilder::<Complex32>::sin(480.0, 48_000.0).build();
    let noise =
        Apply::new(|i: &Complex32| i + rand::thread_rng().gen_range(-100..100) as f32 / 50.0);
    let throttle = Throttle::<Complex32>::new(8000.0);
    let fft = Fft::with_options(FFT_SIZE, FftDirection::Forward, true, None);
    let mag = Apply::new(|x: &Complex32| x.norm());

    let head = Head::<f32>::new((FFT_SIZE * 50) as u64);

    let vec_snk = VectorSink::<f32>::new(FFT_SIZE * 20);

    connect!(fg, src  > noise > fft > throttle >  mag > head > vec_snk);

    let fg = Runtime::new().run(fg)?;

    let snk = fg.kernel::<VectorSink<f32>>(vec_snk).unwrap();
    let v = snk.items();
    println!("{:#?}", v);

    write_to_file(v);
    Ok(())
}

fn write_to_file(v: &Vec<f32>) {
    let dest_path = std::path::Path::new("src/vector.rs");

    let const_declarations = [
        const_declaration!(pub TEST_DATA = v),
        const_declaration!(pub FFT_SIZE = FFT_SIZE),
    ]
    .join("\n");

    std::fs::write(dest_path, const_declarations).unwrap();
}
