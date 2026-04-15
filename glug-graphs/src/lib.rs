use std::io::Cursor;

use chrono::{NaiveTime, Utc};
use glug_glug_core::database::drinks::DateStats;
use image::{ImageEncoder as _, codecs::png::PngEncoder};
use plotters::{
    backend::{PixelFormat, RGBPixel},
    prelude::*,
    style::register_font,
};

const FONT: &str = "sans-serif";

pub fn register_fonts() {
    register_font(
        FONT,
        FontStyle::Normal,
        include_bytes!("../fonts/Monocraft-otf/Monocraft.otf"),
    )
    .ok()
    .expect("failed to register fonts")
}

const W: usize = 1024;
const H: usize = 600;
pub fn graph(stats: DateStats) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    register_fonts();

    let mut buffer: Vec<u8> = vec![0; W * H * RGBPixel::PIXEL_SIZE];

    {
        let root_area =
            BitMapBackend::with_buffer(&mut buffer, (W as u32, H as u32)).into_drawing_area();

        root_area.fill(&RGBColor(22, 31, 40))?;

        let accent = RGBColor(210, 153, 29);

        let title_style = (FONT, 50).into_font().color(&accent);

        let root_area = root_area
            .margin(40, 10, 10, 10)
            .titled(&stats.y_max.to_string(), title_style)?;

        let target_approx = stats.linear_approx(10_000);
        let time_until = target_approx.date.and_time(NaiveTime::MIN).and_utc() - Utc::now();
        let days_until = time_until.num_days().max(0);

        let caption = if days_until == 1 {
            format!("{days_until} day remains")
        } else {
            format!("{days_until} days remain")
        };

        let mut cc = ChartBuilder::on(&root_area)
            .margin(5)
            .set_label_area_size(LabelAreaPosition::Left, 80)
            .set_label_area_size(LabelAreaPosition::Bottom, 40)
            .caption(caption, (FONT, 24).with_color(WHITE))
            .build_cartesian_2d(stats.x_min..target_approx.date, 0..10_000u32)?;

        cc.configure_mesh()
            .axis_style(WHITE.mix(0.5))
            .label_style((FONT, 16).with_color(WHITE))
            .x_labels(11)
            .y_labels(10)
            .disable_mesh()
            .x_label_formatter(&|v| v.format("%m/%Y").to_string())
            .y_label_formatter(&|v| format!("{:.1}", v))
            .draw()?;

        cc.draw_series(DashedLineSeries::new(
            [
                (stats.x_min, stats.y_min),
                (target_approx.date, target_approx.drinks_total),
            ],
            10,
            15,
            ShapeStyle {
                color: WHITE.mix(0.75),
                filled: false,
                stroke_width: 1,
            },
        ))?;

        cc.draw_series(AreaSeries::new(
            stats.stats.iter().map(|x| (x.date, x.drinks_total)),
            0,
            accent,
        ))?;

        // To avoid the IO failure being ignored silently, we manually call the present function
        root_area.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
    }

    let mut png_buffer: Vec<u8> = Vec::new();
    PngEncoder::new(Cursor::new(&mut png_buffer)).write_image(
        &buffer,
        W as u32,
        H as u32,
        image::ExtendedColorType::Rgb8,
    )?;

    Ok(png_buffer)
}
