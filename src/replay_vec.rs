use crate::{vector, MainWindow};

pub async fn wait_for_samples(window_weak: slint::Weak<MainWindow>) {
    use crate::rendering::render_plot;
    use std::time::Duration;
    loop {
        let window_weak = window_weak.clone();

        for values in vector::TEST_DATA.chunks_exact(vector::FFT_SIZE) {
            let window_weak = window_weak.clone();
            window_weak
                .upgrade_in_event_loop(move |app| {
                    if app.get_plot_enable() {
                        app.set_plot_frame(render_plot(&values, &app))
                    }
                })
                .unwrap();

            #[cfg(not(target_arch = "wasm32"))]
            tokio::runtime::Runtime::new()
                .unwrap()
                .spawn(async { tokio::time::sleep(Duration::from_millis(100)).await })
                .await
                .unwrap();
            #[cfg(target_arch = "wasm32")]
            gloo_timers::future::sleep(Duration::from_millis(200)).await; // Replay a little slower on the browser
        }
    }
}
