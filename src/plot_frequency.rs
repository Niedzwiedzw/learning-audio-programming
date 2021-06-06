use std::path::PathBuf;

use crate::pythagorean_chords::{sine_wave, AMPLITUDE, SAMPLE_RATE};
use itertools::Itertools;
use plotters::prelude::*;
use rand::SeedableRng;
use rand_distr::{Distribution, Normal};
use rand_xorshift::XorShiftRng;
use rustfft::{
    num_complex::{self, Complex},
    FftPlanner,
};

use crate::lo_pass_filter::{LoPassFilter, SignalFilter};

use rayon::prelude::*;
use uuid::Uuid;

fn open_in_browser<T: AsRef<std::path::Path>>(file: T) {
    let path = PathBuf::from(file.as_ref());
    std::process::Command::new("firefox")
        .arg(path.into_os_string())
        .status()
        .expect("firefox not in path?");
}

fn plot_histogram(
    data: Vec<(f32, f32, f32)>,
    label: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let uuid = Uuid::new_v4().to_hyphenated().to_string();
    let path: PathBuf = PathBuf::from(format!("/tmp/{}-{}.png", label, uuid));
    {
        let root = BitMapBackend::new(&path, (1920, 1080)).into_drawing_area();

        root.fill(&WHITE.mix(0.88))?;
        let root = root.titled(label, ("sans-serif", 60))?;

        // let areas = root.split_by_breakpoints([944], [80]);
        let min_x = 0.0f32;
        let max_x = data
            .iter()
            .map(|(x, _y, _v)| *x)
            .max_by_key(|v| (*v * 1000.0f32) as usize)
            .expect("input is empty");
        let min_y = data
            .iter()
            .map(|(_x, y, _v)| *y)
            .min_by_key(|v| (*v * 1000.0f32) as usize)
            .expect("input is empty");
        let max_y = data
            .iter()
            .map(|(_x, y, _v)| *y)
            .max_by_key(|v| (*v * 1000.0f32) as usize)
            .expect("input is empty");

        let min_val = data
            .iter()
            .map(|(_x, _y, val)| *val)
            .min_by_key(|v| (*v * 1000.0f32) as usize)
            .expect("input is empty");
        let max_val = data
            .iter()
            .map(|(_x, _y, val)| *val)
            .max_by_key(|v| (*v * 1000.0f32) as usize)
            .expect("input is empty");

        let mut scatter_ctx = ChartBuilder::on(&root)
            .x_label_area_size(40)
            .y_label_area_size(80)
            .build_cartesian_2d(min_x..max_x, min_y..max_y)?;
        scatter_ctx
            .configure_mesh()
            .draw()?;
        scatter_ctx.draw_series(data.iter().map(|(x, y, size)| {
            Circle::new((*x, *y), 2.0, GREEN.mix((size / max_val) as f64).filled())
        }))?;
        root.present()?;
    }
    Ok(path)
}

fn plot(
    data: Vec<f32>,
    label: &str,
    x_values: Vec<f32>,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let uuid = Uuid::new_v4().to_hyphenated().to_string();
    let path: PathBuf = PathBuf::from(format!("/tmp/{}-{}.png", label, uuid));
    {
        let root = BitMapBackend::new(&path, (1920, 1080)).into_drawing_area();
        root.fill(&WHITE)?;
        let max = *data
            .iter()
            .max_by_key(|v| (*v * 1000.0f32) as i64)
            .ok_or("empty data".to_string())?;
        let min = *data
            .iter()
            .min_by_key(|v| (*v * 1000.0f32) as i64)
            .ok_or("empty data".to_string())?;

        let mut chart = ChartBuilder::on(&root)
            .caption(label, ("sans-serif", 50).into_font())
            .margin(5)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(
                x_values.first().expect("range cannot be empty").clone()
                    ..x_values.last().expect("range cannot be empty").clone(),
                (min - min * 0.1)..(max + max * 0.1),
            )?;

        chart.configure_mesh().draw()?;

        chart
            .draw_series(LineSeries::new(
                x_values.into_iter().zip(data.into_iter()),
                &RED,
            ))?
            .label(label)
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?;
        root.present()?;
    }
    Ok(path)
}

fn plot_display(
    data: Vec<f32>,
    label: &str,
    x_values: Vec<f32>,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = plot(data, label, x_values)?;
    open_in_browser(file);
    Ok(())
}

fn frequency_distribution(start: f32, end: f32, step: f32) -> Vec<f32> {
    (0..)
        .take(((end - start) / step) as usize)
        .map(|v| v as f32)
        .map(|v| v * step + start)
        .collect()
}

#[test]
fn test_frequency_distribution() {
    assert_eq!(frequency_distribution(0.0, 0.3, 0.1), vec![0.0, 0.1, 0.2]);
    assert_eq!(
        frequency_distribution(-0.2, 0.1, 0.1),
        vec![-0.2, -0.1, 0.0]
    );
}

fn fft_shift<T: Clone>(values: Vec<T>) -> Vec<T> {
    let (left, right) = values.split_at((values.len() as f32 / 2.0).round() as usize);
    right.into_iter().chain(left.into_iter()).cloned().collect()
}

#[test]
fn test_fft_shift() {
    assert_eq!(fft_shift(vec![0.0, 1.0, 2.0]), vec![2.0, 0.0, 1.0]);
    assert_eq!(
        fft_shift(vec![0., 1., 2., 3., 4., -5., -4., -3., -2., -1.]),
        vec![-5., -4., -3., -2., -1., 0., 1., 2., 3., 4.]
    );
}

pub fn fft_of(buffer: &Vec<f32>) -> Vec<Complex<f32>> {
    let mut sine = buffer
        .iter()
        .cloned()
        .map(|v| Complex { re: v, im: 0.0f32 })
        .collect::<Vec<_>>();
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(sine.len());
    fft.process(&mut sine);
    fft_shift(sine)
}

fn continous_fft_of(buffer: &Vec<f32>, window: usize) -> Vec<(f32, f32, f32)> {
    (0..buffer.len())
        .map(|index| {
            buffer[(index.checked_sub(window).unwrap_or(0)..index)]
                .iter()
                .cloned()
                .collect::<Vec<_>>()
        })
        .enumerate()
        .map(|(index, buffer)| (index as f32, buffer))
        .step_by(window / 2)
        .par_bridge()
        .into_par_iter()
        .filter_map(|(index, buffer)| {
            if buffer.len() == 0 {
                None
            } else {
                Some(
                    fft_of(&buffer)
                        .into_iter()
                        .enumerate()
                        .map(|(freq, c)| (index, ((freq as f32) - (window as f32 / 2.0)) * 44100.0 / window as f32, (c.re.powi(2) + c.im.powi(2)).sqrt()))
                        .collect::<Vec<_>>(),
                )
            }
        })
        .flatten()
        .collect()
}

fn wav_as_f32<T: AsRef<std::path::Path>>(path: &T) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let mut reader = hound::WavReader::open(path)?;
    let samples = reader.samples::<i32>().collect::<Result<Vec<i32>, _>>()?;
    let samples = samples.into_iter().map(|v| v as f32).step_by(2).collect();
    Ok(samples)
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("4 :: plotting frequency");

    let sine = (0..(1024 * 100))
        .map(|v| v as f32)
        .map(|t| 0.11 * 2.0 * std::f32::consts::PI * t)
        .map(f32::sin)
        .collect::<Vec<_>>();
    plot_display(sine.clone(), "sine", (0..100).map(|v| v as f32).collect())?;
    let sine = sine.into_iter().collect_vec();
    let frequencies = frequency_distribution(-0.5, 0.5, 1.0 / 102400.0);
    plot_display(
        fft_of(&sine)
            .into_iter()
            .map(|c| (c.re.powi(2) + c.im.powi(2)).sqrt())
            .collect(),
        "FFT magnitude",
        frequencies.clone(),
    )?;
    plot_display(
        fft_of(&sine)
            .clone()
            .into_iter()
            .map(|c| c.re.atan2(c.im))
            .collect(),
        "FFT phase",
        frequencies,
    )?;

    // histogram

    // let sd = 0.13;
    // let data: Vec<(f32, f32)> = {
    //     let norm_dist = Normal::new(0.5, sd).unwrap();
    //     let mut x_rand = XorShiftRng::from_seed(*b"MyFragileSeed123");
    //     let mut y_rand = XorShiftRng::from_seed(*b"MyFragileSeed321");
    //     let x_iter = norm_dist.sample_iter(&mut x_rand);
    //     let y_iter = norm_dist.sample_iter(&mut y_rand);
    //     x_iter.zip(y_iter).take(5000).collect()
    // };

    // histogram of frequencies for a sinewave
    open_in_browser(plot_histogram(
        continous_fft_of(&sine, 128),
        "histogram of continous fft",
    )?);

    open_in_browser(plot_histogram(
        continous_fft_of(
            &wav_as_f32(&std::path::PathBuf::from(crate::lo_pass_filter::INPUT_FILE))?,
            2048,
        ),
        "histogram of continous fft for actual WAV audio",
    )?);
    let niedzwiedz_substance =
        wav_as_f32(&std::path::PathBuf::from("data/niedzwiedz-substance.wav"))?;
    plot_display(
        niedzwiedz_substance.clone(),
        "Niedźwiedz - Substance",
        (0..niedzwiedz_substance.len()).map(|v| v as f32).collect(),
    )?;

    let buffer_size = 4096;
    let task_name = format!("FFT WIDTH - {}", buffer_size);
    open_in_browser(plot_histogram(
        continous_fft_of(&niedzwiedz_substance, buffer_size),
        &task_name,
    )?);

    let task_name = format!("LO PASS - {}", 1);
    open_in_browser(plot_histogram(
        continous_fft_of(
            &LoPassFilter::new(niedzwiedz_substance.clone().into_iter().map(|v| v as i32), 1)
                .map(|v| v as f32)
                .collect(),
            buffer_size,
        ),
        &task_name,
    )?);
    let task_name = format!("LO PASS - {}", 2);
    open_in_browser(plot_histogram(
        continous_fft_of(
            &LoPassFilter::new(niedzwiedz_substance.into_iter().map(|v| v as i32), 2)
                .map(|v| v as f32)
                .collect(),
            buffer_size,
        ),
        &task_name,
    )?);

    Ok(())
}
