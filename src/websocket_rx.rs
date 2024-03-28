use crate::{rendering::render_plot, MainWindow};
use futures_util::StreamExt;
use log::info;
use slint::Weak;
use std::time::Duration;
use tokio_tungstenite_wasm::{connect, Message};

pub async fn wait_for_samples(window_weak: Weak<MainWindow>) {
    #[cfg(not(target_arch = "wasm32"))]
    tokio::runtime::Runtime::new()
        .unwrap()
        .spawn(async { websocket_rx(window_weak).await })
        .await
        .unwrap();

    #[cfg(target_arch = "wasm32")]
    websocket_rx(window_weak).await;
}

async fn websocket_rx(window_weak: Weak<MainWindow>) -> ! {
    loop {
        info!("Waiting for samples...");

        // This "connect" causes an "Uncaught Error: closure invoked recursively or after being dropped"
        // when compiled to WASM. However, that does not seem to be a problem.
        if let Ok(socket) = connect("ws://localhost:9001/").await {
            let (_, mut read) = socket.split();

            while let Some(message) = read.next().await {
                let values: Vec<f32> = match message {
                    Ok(Message::Binary(data)) => data
                        .chunks_exact(4)
                        .map(|f| f32::from_le_bytes([f[0], f[1], f[2], f[3]]))
                        .collect(),
                    _ => vec![0.0, 0.0, 0.0],
                };

                window_weak
                    .upgrade_in_event_loop(move |app| {
                        if app.get_plot_enable() {
                            app.set_plot_frame(render_plot(&values, &app))
                        }
                    })
                    .unwrap();
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        tokio::time::sleep(Duration::from_millis(100)).await;
        #[cfg(target_arch = "wasm32")]
        gloo_timers::future::sleep(Duration::from_millis(100)).await;
    }
}
