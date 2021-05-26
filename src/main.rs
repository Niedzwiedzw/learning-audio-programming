#![feature(min_type_alias_impl_trait)]

mod generate_sinewave;
mod pythagorean_chords;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    generate_sinewave::run()?;
    pythagorean_chords::run()?;
    Ok(())
}
