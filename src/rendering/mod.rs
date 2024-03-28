use crate::{MainWindow, PlotMeta};
use log::{debug, trace};
use plotters::prelude::*;
use slint::{ComponentHandle, Image};

// This is a bitmap backend without text for WASM compatibility
#[cfg(all(target_arch = "wasm32", not(feature = "svg")))]
mod wasm_backend;

const PLOT_LINE_COLOR: RGBColor = RGBColor(0, 175, 255);

/// Render the FFT `data`  into an [`Image`]
pub fn render_plot(data: &[f32], app: &MainWindow) -> Image {
    debug!("Start Plotting");

    // Obtain dimensions of the plot image
    let width = app.get_plot_width() as u32;
    let height = app.get_plot_height() as u32;

    assert!(width > 0, "Width must be >0");
    assert!(height > 0, "Height must be >0");

    trace!("width, height = {}, {}", width, height);

    // Compute the y axis min and max values
    let data_y_min = data.iter().cloned().reduce(f32::min).unwrap().ceil();
    let data_y_max = data.iter().cloned().reduce(f32::max).unwrap().floor();

    // Store to global metadata for (potential) later use
    app.global::<PlotMeta>().set_min_value(data_y_min);
    app.global::<PlotMeta>().set_max_value(data_y_max);

    // Set the y-axis limits either automatically or to the user-defined value
    let (y_min, y_max) = if app.get_y_auto_update() {
        (data_y_min, data_y_max)
    } else {
        (app.get_y_axis_min() as f32, app.get_y_axis_max() as f32)
    };

    // x-axis limits set to match how `enumerate` generates the x values later on
    let x_min = 0;
    let x_max = data.len() - 1;

    // Initialize a backend depending on the features
    #[cfg(not(feature = "svg"))]
    let mut pixel_buffer = slint::SharedPixelBuffer::new(width, height);
    #[cfg(not(feature = "svg"))]
    let backend = BitMapBackend::with_buffer(pixel_buffer.make_mut_bytes(), (width, height));

    // Plotters requires TrueType fonts from the file system to draw axis text - we skip that for
    // WASM for now
    #[cfg(all(target_arch = "wasm32", not(feature = "svg")))]
    let backend = wasm_backend::BackendWithoutText { backend };

    #[cfg(feature = "svg")]
    let mut svg_string_buffer = String::new();
    #[cfg(feature = "svg")]
    let backend = plotters_svg::SVGBackend::with_string(&mut svg_string_buffer, (width, height));

    let root = backend.into_drawing_area();
    root.fill(&WHITE).expect("error filling drawing area");

    // Build a 2D chart
    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(28)
        .y_label_area_size(28)
        .margin(20)
        .build_cartesian_2d(
            (x_min as f64)..(x_max as f64),
            (y_min as f64)..(y_max as f64),
        )
        .expect("failed to build chart");

    // Configure the chart
    chart
        .configure_mesh()
        //.disable_mesh() // Disable mesh for faster SVG rendering
        .bold_line_style(BLUE.mix(0.1))
        .light_line_style(BLUE.mix(0.05))
        .axis_style(ShapeStyle::from(BLUE.mix(0.45)).stroke_width(1))
        .x_labels(20)
        .x_label_style(("sans-serif", 15).into_font().color(&BLUE.mix(0.65)))
        .x_label_formatter(&|x| format!("{}", x))
        .y_labels(10)
        .y_label_style(("sans-serif", 15).into_font().color(&BLUE.mix(0.65)))
        .y_label_formatter(&|y| format!("{}", y))
        .draw()
        .expect("failed to draw chart mesh");

    // Define the actual data series to be plotted
    let area_series = AreaSeries::new(
        data.iter().enumerate().map(|(x, y)| (x as f64, *y as f64)),
        -1.0,
        PLOT_LINE_COLOR.mix(0.175),
    )
    // Setting to 2 can cause plotters to hang see https://github.com/plotters-rs/plotters/issues/562
    .border_style(ShapeStyle::from(PLOT_LINE_COLOR).stroke_width(1));

    // Draw the defined data series onto the chart
    chart
        .draw_series(area_series)
        .expect("failed to draw chart data");

    // Call explicitly to avoid errors being ignored on dropping
    root.present().expect("error presenting");
    drop(chart);
    drop(root);

    debug!("Done Rendering. Sending to GUI.");

    #[cfg(not(feature = "svg"))]
    {
        Image::from_rgb8(pixel_buffer)
    }
    #[cfg(feature = "svg")]
    {
        Image::load_from_svg_data(svg_string_buffer.as_bytes()).unwrap()
    }
}
