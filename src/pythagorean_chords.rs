use hound; // WAV codec library
use itertools::Itertools;

const SAMPLE_RATE: u32 = 44100; // 44100 signal samples per second
const AMPLITUDE: f32 = std::i16::MAX as f32 * 0.6; // to avoid distortion

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

fn sum_iters<'a>(
    one: impl Iterator<Item = f32> + 'a,
    other: impl Iterator<Item = f32> + 'a,
) -> Box<dyn Iterator<Item = f32> + 'a> {
    Box::new(
        one.zip(other)
            .map(|(one_val, other_val)| one_val + other_val),
    )
}

fn chord(frequencies: Vec<f32>) -> impl Iterator<Item = f32> {
    let sines = frequencies.into_iter().map(|freq| sine_wave(freq));
    let final_wave = sines.fold(Box::new(std::iter::repeat(0.0f32)) as _, sum_iters);
    Box::new(final_wave)
}

mod pythagorean {
    use super::A4;
    const OFFSETS: [f32; 12] = [
        1.0 / 1.0,
        256.0 / 243.0,
        9.0 / 8.0,
        32.0 / 27.0,
        81.0 / 64.0,
        4.0 / 3.0,
        // 1024.0/729.0,
        729.0 / 512.0,
        3.0 / 2.0,
        128.0 / 81.0,
        27.0 / 16.0,
        16.0 / 9.0,
        243.0 / 128.0,
    ];

    pub fn notes() -> Vec<f32> {
        OFFSETS.iter().map(|offset| A4 * offset).collect()
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut song_chords: Vec<Vec<f32>> = vec![

    ];

    let mut notes = pythagorean::notes();
    notes.append(
        &mut notes
            .iter()
            .map(|v| v * 2.0)
            .chain(notes.iter().map(|v| v * 4.0))

            .chain(notes.iter().map(|v| v * 8.0))
            .collect(),
    );
    let scale = vec![
        notes[0],
        notes[2],
        notes[4],
        notes[5],
        notes[7],
        notes[9],
        notes[11],
        notes[12 + 0],
        notes[12 + 2],
        notes[12 + 4],
        notes[12 + 5],
        notes[12 + 7],
        notes[12 + 9],
        notes[12 + 11],
        notes[2 * 12 + 0],
        notes[2 * 12 + 2],
        notes[2 * 12 + 4],
        notes[2 * 12 + 5],
        notes[2 * 12 + 7],
        notes[2 * 12 + 9],
        notes[2 * 12 + 11],

        notes[3 * 12 + 0],
        notes[3 * 12 + 2],
        notes[3 * 12 + 4],
        notes[3 * 12 + 5],
        notes[3 * 12 + 7],
        notes[3 * 12 + 9],
        notes[3 * 12 + 11],
    ];

    #[rustfmt::skip]
    let barka = vec![
        9, 9, 9, // pan
        9, 9, 9,
        9, 8, 9, // kiedyś
        10, 9, 8, // stanął nad
        7, 7, 7, // brze-e-giem
        7, 7, 7,
        7, 7, 7,
        8, 8, 9,

        10, 10, 10,
        10, 10, 10,
        10, 10, 10,
        10, 10, 9,
        8, 8, 8,
        8, 8, 8,
        8, 8, 4,
        7, 7, 8,

        9, 9, 9,
        9, 9, 9,
        9, 9, 9,
        10, 10, 8,
        7, 7, 7,
        7, 7, 7,
        7, 7, 7,
        7, 7, 7,

        12, 12, 12,
        12, 12, 12,
        12, 12, 13,
        14, 13, 12,
        11, 11, 11,
        11, 11, 11,
        11, 11, 11,
        10, 10, 9,

        10, 10, 10,
        10, 10, 10,
        10, 10, 11,
        12, 11, 10,
        9, 9, 9,
        9, 9, 9,
        9, 9, 9,
        7, 7, 7,

        12, 12, 12,
        12, 12, 12,
        12, 12, 13,
        14, 13, 12,
        11, 11, 11,
        9, 9, 9,
        9, 9, 9,
        10, 10, 9,

        10, 10, 10,
        10, 10, 10,
        10, 8, 9,
        10, 9, 8,
        7, 7, 7,
        7, 7, 7,
        7, 7, 7,
        7, 7, 7,
    ];
    song_chords.append(
        &mut barka
            .into_iter()
            .map(|v| vec![scale[v], scale[v + 2]])
            .collect(),
    );

    let note_length = 0.15 * SAMPLE_RATE as f32;
    let song = song_chords.into_iter().map(chord).fold(
        Box::new(std::iter::empty::<f32>()) as _,
        |song: Box<dyn Iterator<Item = f32>>, chunk| {
            Box::new(
                song.chain(chunk.take(note_length as usize))
                    .chain(std::iter::repeat(0.0f32).take((note_length * 0.1) as usize)),
            )
            // 2 seconds of each chord
        },
    );
    println!("2 :: generating pythagorean chords");
    let spec = hound::WavSpec {
        channels: 1, // mono
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create("./output/pythagorean_chords.wav", spec)?;
    for sample in song {
        writer.write_sample(sample as i16)?;
    }
    Ok(())
}
