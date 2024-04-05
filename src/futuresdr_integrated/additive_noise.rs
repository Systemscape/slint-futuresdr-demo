use futuresdr::{
    anyhow::Result,
    log::trace,
    macros::{async_trait, message_handler},
    num_complex::Complex32,
    runtime::{
        Block, BlockMeta, BlockMetaBuilder, Kernel, MessageIo, MessageIoBuilder, Pmt, StreamIo,
        StreamIoBuilder, WorkIo,
    },
};
use rand::Rng;

/// Add noise to the real and imaginary part of `Complex32` samples.
///
/// # Inputs
/// `in`: Input
///
/// # Outputs
/// `out`: Noise-corrupted outputs
///
/// # Gain
/// The gain controls from which values the added noise is "drawn" by the RNG:
///
/// ```norun
/// let noise =  Complex32::new(
///     rand::thread_rng().gen_range(-self.gain..self.gain),
///     rand::thread_rng().gen_range(-self.gain..self.gain),
///  );
pub struct AdditiveNoise {
    gain: f32,
}

impl AdditiveNoise {
    pub fn new(gain: f32) -> Block {
        Block::new(
            BlockMetaBuilder::new("AdditiveNoise").build(),
            StreamIoBuilder::new()
                .add_input::<Complex32>("in")
                .add_output::<Complex32>("out")
                .build(),
            MessageIoBuilder::<Self>::new()
                .add_input("gain", Self::gain_handler)
                .build(),
            Self { gain },
        )
    }

    #[message_handler]
    fn gain_handler(
        &mut self,
        _io: &mut WorkIo,
        _mio: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
        p: Pmt,
    ) -> Result<Pmt> {
        match &p {
            Pmt::F32(v) => self.gain = *v,
            Pmt::F64(v) => self.gain = *v as f32,
            Pmt::U32(v) => self.gain = *v as f32,
            Pmt::U64(v) => self.gain = *v as f32,
            Pmt::Null => self.gain = 0.0,
            _ => return Ok(Pmt::InvalidValue),
        };

        Ok(Pmt::Ok)
    }
}

#[doc(hidden)]
#[async_trait]
impl Kernel for AdditiveNoise {
    async fn work(
        &mut self,
        io: &mut WorkIo,
        sio: &mut StreamIo,
        _mio: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let i = sio.input(0).slice::<Complex32>();
        let o = sio.output(0).slice::<Complex32>();

        let m = std::cmp::min(i.len(), o.len());
        if m > 0 {
            for (v, r) in i.iter().zip(o.iter_mut()) {
                let noise = Complex32::new(
                    rand::thread_rng().gen_range(-self.gain..self.gain),
                    rand::thread_rng().gen_range(-self.gain..self.gain),
                );
                trace!("Add noise: {}", noise);
                *r = v + noise;
            }

            sio.input(0).consume(m);
            sio.output(0).produce(m);
        }

        if sio.input(0).finished() && m == i.len() {
            io.finished = true;
        }

        Ok(())
    }
}
