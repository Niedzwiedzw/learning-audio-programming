use hound; // WAV codec library
use itertools::Itertools;

const SAMPLE_RATE: u32 = 44100; // 44100 signal samples per second
const AMPLITUDE: f32 = std::f32::MAX * 0.66; // to avoid distortion

const A4: f32 = 440.0;

fn sine_wave(freq: f32) -> impl Iterator<Item = f32> {
    Box::new(
        std::iter::repeat(())
            .enumerate()
            .map(|(index, _)| index)
            .map(|v| v as f32 / SAMPLE_RATE as f32)
            .map(move |v| (v * freq * 2.0 * std::f32::consts::PI).sin() * AMPLITUDE),
    )
}

fn sum_iters(
    one: Box<dyn Iterator<Item = f32>>,
    other: impl Iterator<Item = f32>,
) -> Box<dyn Iterator<Item = f32>> {
    Box::new(
        one.zip(other)
            .map(|(one_val, other_val)| one_val + other_val),
    )
}

fn chord(frequencies: Vec<f32>) -> impl Iterator<Item = f32> {
    let sines = frequencies.into_iter().map(|freq| sine_wave(freq));
    let final_wave = sines
        .fold(Box::new(std::iter::repeat(0.0f32)), sum_iters);
    Box::new(final_wave)
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    sine_wave(A4).zip(sine_wave(A4 * 2.0));
    Ok(())
}
