use futuresdr::{
    anyhow::Result,
    futures::channel::mpsc::Sender,
    macros::async_trait,
    runtime::{
        Block, BlockMeta, BlockMetaBuilder, Kernel, MessageIo, MessageIoBuilder, StreamIo,
        StreamIoBuilder, WorkIo,
    },
};

/// Send vector of samples from a Flowgraph into a channel.
///
/// # Inputs
///
/// `in`: Samples retrieved from the flowgraph
pub struct ChannelSink<const FFT_SIZE: usize> {
    tx: Sender<Box<[f32; FFT_SIZE]>>,
}

impl<const FFT_SIZE: usize> ChannelSink<FFT_SIZE> {
    pub fn new(tx: Sender<Box<[f32; FFT_SIZE]>>) -> Block {
        Block::new(
            BlockMetaBuilder::new("ChannelSink").build(),
            StreamIoBuilder::new().add_input::<f32>("in").build(),
            MessageIoBuilder::<Self>::new().build(),
            Self { tx },
        )
    }
}

#[doc(hidden)]
#[async_trait]
impl<const FFT_SIZE: usize> Kernel for ChannelSink<FFT_SIZE> {
    async fn work(
        &mut self,
        io: &mut WorkIo,
        sio: &mut StreamIo,
        _mio: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let i = sio.input(0).slice::<f32>();

        if sio.input(0).finished() {
            io.finished = true;
        }

        let n = i.len() / FFT_SIZE;
        if n > 0 {
            let mut a = [0.0; FFT_SIZE];
            a.copy_from_slice(&i[(n - 1) * FFT_SIZE..n * FFT_SIZE]);
            sio.input(0).consume(n * FFT_SIZE);
            let _ = self.tx.try_send(Box::new(a));
        }

        Ok(())
    }
}
