use hound; // WAV codec library

const SAMPLE_RATE: u32 = 44100;  // 44100 signal samples per second

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("1 :: generating sinewave");
    let spec = hound::WavSpec {
        channels: 1, // mono
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create("./output/generate_sinewave.wav", spec)?;
    let length = 5; // seconds
    let sample_length = length * SAMPLE_RATE;
    for t in (0..(sample_length)).map(|x| x as f32 / SAMPLE_RATE as f32) {
        let sample = (t * 440.0 * 2.0 * std::f32::consts::PI).sin();
        let amplitude = std::i16::MAX as f32;
        writer.write_sample((sample * amplitude) as i16)?;
    }
    Ok(())
}
