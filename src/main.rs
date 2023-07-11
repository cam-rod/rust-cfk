mod temp;

use clap::Parser;

use temp::Temp;

#[derive(Parser, Debug)]
#[command(about)]
/// A script to convert between Celsius, Fahrenheit, and Kelvin.
struct Unit {
    #[arg(id = "original")]
    /// The original value, provided as a number-letter combo (ex. 32F, 0C, 273K)
    original: Temp,

    #[arg(id = "unit")]
    /// The unit to convert into
    new_unit: char
}

fn main() {
    let args = Unit::parse();

    let result = match args.new_unit.to_ascii_uppercase() {
        'C' => Ok(args.original.to_celsius()),
        'F' => Ok(args.original.to_fahrenheit()),
        'K' => Ok(args.original.to_kelvin()),
        _ => Err("Failed: {args.new_unit} is not a valid temperature unit.")
    };

    match result {
        Ok(new_temp) => println!("{} is equal to {}", args.original, new_temp),
        Err(msg) => eprintln!("{msg}")
    }
}