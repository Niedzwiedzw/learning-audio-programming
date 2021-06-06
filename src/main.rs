#![feature(min_type_alias_impl_trait)]

mod generate_sinewave;
mod pythagorean_chords;
mod lo_pass_filter;
mod plot_frequency;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    generate_sinewave::run()?;
    pythagorean_chords::run()?;
    lo_pass_filter::run()?;
    plot_frequency::run()?;
    Ok(())
}
