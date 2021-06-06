use hound::{WavReader, WavWriter};
use itertools::{Itertools, Tee};
use ringbuf::{Consumer, Producer, RingBuffer};

pub const INPUT_FILE: &str = "./data/audio-input.wav";
const OUTPUT_FILE_LO_PASS: &str = "./output/after-lo-pass.wav";
const OUTPUT_FILE_HI_PASS: &str = "./output/after-hi-pass.wav";

const I24_MAX: i32 = (1 << 23) - 1;
const I24_MIN: i32 = -I24_MAX;


pub trait SignalFilter<T: Iterator<Item = i32>>: Sized + Iterator<Item = i32> {
    fn new(input: T, width: i32) -> Self;
}

struct HiPassFilter<T: Iterator<Item = i32>> {
    lo_pass: LoPassFilter<Tee<T>>,
    dry: Tee<T>,
}

impl<T: Iterator<Item = i32>> Iterator for HiPassFilter<T> {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.dry.next()? - self.lo_pass.next()?)
    }
}

impl<T: Iterator<Item = i32>> SignalFilter<T> for HiPassFilter<T> {
    fn new(input: T, width: i32) -> Self {
        let (one, two) = input.tee();
        let lo_pass = LoPassFilter::new(one, width);
        Self {
            lo_pass,
            dry: two,
        }
    }
}

pub struct LoPassFilter<T: Iterator<Item = i32>> {
    pub input: T,
    previous: i32,
    width: i32,
}

impl<T: Iterator<Item = i32>>  SignalFilter<T> for LoPassFilter<T> {
    fn new(input: T, width: i32) -> Self {
        let previous = 0;

        Self {
            input,
            previous,
            width,
        }
    }
}

#[inline]
fn weighted_average(one: i32, other: i32, ratio: f64) -> i32 {
    debug_assert!(ratio > 0.0);
    debug_assert!(ratio < 1.0);
    ((one as f64) * ratio + (other as f64) * (1.0 - ratio)) as i32
}

fn one_minus(sample: i32) -> i32 {
    match sample.is_positive() {
        true => I24_MAX - sample,
        false => I24_MIN - sample,
    }
}

impl<T: Iterator<Item = i32>> Iterator for LoPassFilter<T> {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        let next = weighted_average(self.input.next()?, self.previous, 0.1f64.powi(self.width));
        self.previous = next;
        Some(next)
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error
                               >> {
    println!("3 :: applying a lo pass filter");
    let mut reader = hound::WavReader::open(INPUT_FILE)?;
    let spec = reader.spec();
    println!("{:#?}", spec);
    let samples = reader.samples::<i32>().collect::<Result<Vec<i32>, _>>()?;
    let left = samples.iter().enumerate().filter(|(index, _value)| index % 2 == 0).map(|(_i, v)| *v).collect::<Vec<_>>();
    let right = samples.iter().enumerate().filter(|(index, _value)| index % 2 == 1).map(|(_i, v)| *v).collect::<Vec<_>>();
    let left_filter = LoPassFilter::new(left.into_iter(), 2);
    let right_filter = LoPassFilter::new(right.into_iter(), 2);

    let mut writer = hound::WavWriter::create(OUTPUT_FILE_LO_PASS, spec)?;
    for (l, r) in  left_filter.into_iter().zip(right_filter.into_iter()) {
        writer.write_sample(l)?;
        writer.write_sample(l)?;
    }

    println!("4 :: applying a hi pass filter");
    let mut reader = hound::WavReader::open(INPUT_FILE)?;
    let spec = reader.spec();
    println!("{:#?}", spec);
    let samples = reader.samples::<i32>().collect::<Result<Vec<i32>, _>>()?;
    let left = samples.iter().enumerate().filter(|(index, _value)| index % 2 == 0).map(|(_i, v)| *v).collect::<Vec<_>>();
    let right = samples.iter().enumerate().filter(|(index, _value)| index % 2 == 1).map(|(_i, v)| *v).collect::<Vec<_>>();
    let left_filter = HiPassFilter::new(left.into_iter(), 2);
    let right_filter = HiPassFilter::new(right.into_iter(), 2);

    let mut writer = hound::WavWriter::create(OUTPUT_FILE_HI_PASS, spec)?;
    for (l, r) in  left_filter.into_iter().zip(right_filter.into_iter()) {
        writer.write_sample(l)?;
        writer.write_sample(l)?;
    }

    Ok(())
}
