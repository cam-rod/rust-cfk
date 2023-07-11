use std::fmt::{Display, Formatter};
use std::str::FromStr;

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

const CEL: TempUnit = TempUnit('C');
const FAH: TempUnit = TempUnit('F');
const KEL: TempUnit = TempUnit('K');
const CONV_ERROR_MSG: &str =
    "Yikes! Seems you manually created this temperature, since we can't convert it";

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TempUnit(char);

impl TryFrom<char> for TempUnit {
    type Error = String;

    fn try_from(unit: char) -> Result<Self, Self::Error> {
        [CEL.0, FAH.0, KEL.0]
            .contains(&unit.to_ascii_uppercase())
            .then_some(Self(unit))
            .ok_or(format!("{unit} is not a valid temperature unit"))
    }
}

impl From<TempUnit> for char {
    fn from(unit: TempUnit) -> Self {
        unit.0
    }
}

impl Display for TempUnit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
/// A representation of a temperature, in C, F, or K
pub struct Temp {
    pub scalar: Decimal,
    pub unit: TempUnit,
}

#[allow(unused)]
impl Temp {
    /// For testing purposes only.
    fn new(scalar: Decimal, unit: TempUnit) -> Self {
        Temp { scalar, unit }
    }

    pub fn to_celsius(self) -> Self {
        match self.unit {
            CEL => self,
            FAH => Self {
                scalar: (self.scalar - dec!(32)) * (dec!(5) / dec!(9)),
                unit: CEL,
            },
            KEL => Self {
                scalar: self.scalar - dec!(273.15),
                unit: CEL,
            },
            _ => panic!("{} to Celsius: {}", CONV_ERROR_MSG, self),
        }
    }

    pub fn to_fahrenheit(self) -> Self {
        match self.unit {
            CEL => Self {
                scalar: self.scalar * dec!(1.8) + dec!(32),
                unit: FAH,
            },
            FAH => self,
            KEL => Self {
                scalar: self.scalar * dec!(1.8) - dec!(459.67),
                unit: FAH,
            },
            _ => panic!("{} to Fahrenheit: {}", CONV_ERROR_MSG, self),
        }
    }

    pub fn to_kelvin(self) -> Self {
        match self.unit {
            CEL => Self {
                scalar: self.scalar + dec!(273.15),
                unit: KEL,
            },
            FAH => Self {
                scalar: (self.scalar + dec!(459.67)) * (dec!(5) / dec!(9)),
                unit: KEL,
            },
            KEL => self,
            _ => panic!("{} to Kelvin: {}", CONV_ERROR_MSG, self),
        }
    }
}

impl Display for Temp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.scalar.normalize(), self.unit)
    }
}

impl FromStr for Temp {
    type Err = String;

    /// Permitted inputs are of the form 32F, 0C, or 273.6K. Other strings will fail.
    fn from_str(temp_str: &str) -> Result<Self, Self::Err> {
        match temp_str.chars().collect::<Vec<char>>().split_last() {
            Some((split_unit, split_scalar)) => {
                let conv_scalar = Decimal::from_str(split_scalar.iter().collect::<String>().trim());
                let conv_unit = TempUnit::try_from(*split_unit);

                if let (Ok(scalar), Ok(unit)) = (&conv_scalar, &conv_unit) {
                    Ok(Self {
                        scalar: *scalar,
                        unit: *unit,
                    })
                } else {
                    Err(format!(
                        "Unable to convert {} into temperature:\n{}\n{}",
                        temp_str,
                        conv_scalar.map_or_else(|err| err.to_string(), |_| String::default()),
                        conv_unit.map_or_else(|err| err, |_| String::default())
                    ))
                }
            }
            None => Err(format!("Invalid temperature value: {temp_str}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Temp;
    use super::{CEL, FAH, KEL};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    #[test]
    fn test_valid_temp() -> Result<(), String> {
        "15C".parse::<Temp>()?;
        "122314234K".parse::<Temp>()?;
        "-347.4f".parse::<Temp>()?;
        Ok(())
    }

    #[test]
    #[should_panic]
    fn test_invalid_tempunit() {
        "15d".parse::<Temp>().unwrap();
    }

    #[test]
    #[should_panic]
    fn test_missing_tempunit() {
        "1234.614".parse::<Temp>().unwrap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_scalar() {
        "123sdafsd23445.4F".parse::<Temp>().unwrap();
    }

    #[test]
    fn test_to_celsius() -> Result<(), String> {
        assert_eq!(
            Temp::new(dec!(32), FAH).to_celsius(),
            Temp::new(dec!(0), CEL)
        );
        assert_eq!(
            Temp::new(dec!(234.63), KEL).to_celsius(),
            Temp::new(dec!(-38.52), CEL)
        );
        assert_eq!(
            Temp::new(dec!(-2345.7), CEL).to_celsius(),
            Temp::new(dec!(-2345.7), CEL)
        );
        Ok(())
    }

    #[test]
    fn test_to_fahrenheit() -> Result<(), String> {
        assert_eq!(
            Temp::new(dec!(-18), CEL).to_fahrenheit(),
            Temp::new(dec!(-0.4), FAH)
        );
        assert_eq!(
            Temp::new(dec!(38653675), KEL).to_fahrenheit(),
            Temp::new(dec!(69576155.33), FAH)
        );
        assert_eq!(
            Temp::new(dec!(12), FAH).to_fahrenheit(),
            Temp::new(dec!(12), FAH)
        );
        Ok(())
    }

    #[test]
    fn test_to_lord_kelvin() -> Result<(), String> {
        assert_eq!(
            Temp::new(dec!(25), CEL).to_kelvin(),
            Temp::new(dec!(298.15), KEL)
        );
        // We love floating point errors
        assert!(
            Decimal::abs(
                &(Temp::new(dec!(-2002), FAH).to_kelvin().scalar
                    - Temp::new(dec!(-856.85), KEL).scalar)
            ) < dec!(0.0001)
        );
        assert_eq!(
            Temp::new(dec!(0.0001), KEL).to_kelvin(),
            Temp::new(dec!(0.0001), KEL)
        );
        Ok(())
    }
}
