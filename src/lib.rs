use std::collections::HashMap;
use std::fmt;

use num_bigint::BigInt;
use num_rational::BigRational;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Stone {
    chance: Chance,
    lines: [u8; 3],
    rolls: [u8; 3],
}

impl Stone {
    pub fn succeed(&self, line: usize) -> Self {
        let mut lines = self.lines;
        let mut rolls = self.rolls;

        lines[line] += 1;
        rolls[line] += 1;

        Self {
            chance: self.chance.succeed(),
            lines,
            rolls,
        }
    }

    pub fn fail(&self, line: usize) -> Self {
        let lines = self.lines;
        let mut rolls = self.rolls;

        rolls[line] += 1;

        Self {
            chance: self.chance.fail(),
            lines,
            rolls,
        }
    }
}

impl fmt::Display for Stone {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.chance)?;
        for line in 0..3 {
            write!(fmt, " | {}/{}", self.lines[line], self.rolls[line])?;
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Chance {
    P25,
    P35,
    P45,
    P55,
    P65,
    P75,
}

impl fmt::Display for Chance {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let chance = match self {
            Chance::P25 => "25",
            Chance::P35 => "35",
            Chance::P45 => "45",
            Chance::P55 => "55",
            Chance::P65 => "65",
            Chance::P75 => "75",
        };

        write!(fmt, "{}%", chance)
    }
}

impl Default for Chance {
    fn default() -> Self {
        Chance::P75
    }
}

impl Chance {
    pub fn succeed(&self) -> Self {
        match self {
            Chance::P25 => Chance::P25,
            Chance::P35 => Chance::P25,
            Chance::P45 => Chance::P35,
            Chance::P55 => Chance::P45,
            Chance::P65 => Chance::P55,
            Chance::P75 => Chance::P65,
        }
    }

    pub fn fail(&self) -> Self {
        match self {
            Chance::P25 => Chance::P35,
            Chance::P35 => Chance::P45,
            Chance::P45 => Chance::P55,
            Chance::P55 => Chance::P65,
            Chance::P65 => Chance::P75,
            Chance::P75 => Chance::P75,
        }
    }

    pub fn to_probability_success(&self) -> BigRational {
        let numerator = match self {
            Chance::P25 => 25,
            Chance::P35 => 35,
            Chance::P45 => 45,
            Chance::P55 => 55,
            Chance::P65 => 65,
            Chance::P75 => 75,
        };

        BigRational::new(BigInt::from(numerator), BigInt::from(100))
    }

    pub fn to_probability_failure(&self) -> BigRational {
        let numerator = match self {
            Chance::P25 => 75,
            Chance::P35 => 65,
            Chance::P45 => 55,
            Chance::P55 => 45,
            Chance::P65 => 35,
            Chance::P75 => 25,
        };

        BigRational::new(BigInt::from(numerator), BigInt::from(100))
    }
}

impl From<usize> for Chance {
    fn from(chance: usize) -> Self {
        match chance {
            0 | 25 => Chance::P25,
            1 | 35 => Chance::P35,
            2 | 45 => Chance::P45,
            3 | 55 => Chance::P55,
            4 | 65 => Chance::P65,
            5 | 75 => Chance::P75,
            value => panic!("Illegal value for `Chance`: {}", value),
        }
    }
}

const ROLLS: u8 = 10;
const GOOD: u8 = 7;
const BAD: u8 = 4;

pub fn evaluate(stone: Stone) -> (usize, BigRational) {
    let mut max_line = 0;
    let mut max_value = BigRational::new(BigInt::from(0u8), BigInt::from(1u8));
    let mut cache = HashMap::new();

    for line in 0..3 {
        let value = recurse_weighted(stone, &mut cache, line);
        if value > max_value {
            max_line = line;
            max_value = value;
        }
    }

    (max_line, max_value)
}

fn recurse_weighted(
    stone: Stone,
    cache: &mut HashMap<Stone, BigRational>,
    line: usize,
) -> BigRational {
    stone.chance.to_probability_success() * recurse(stone.succeed(line), cache)
        + stone.chance.to_probability_failure() * recurse(stone.fail(line), cache)
}

fn recurse(stone: Stone, cache: &mut HashMap<Stone, BigRational>) -> BigRational {
    if let Some(value) = cache.get(&stone) {
        return value.clone();
    }

    dbg!(cache.len());

    // Terminal state: success
    if stone.lines[0] >= GOOD
        && stone.lines[1] >= GOOD
        && stone.rolls[2] == ROLLS
        && stone.lines[2] <= BAD
    {
        return BigRational::new(BigInt::from(1i8), BigInt::from(1i8));
    }

    // Terminal state: failure
    if ROLLS - stone.rolls[0] + stone.lines[0] < GOOD
        || ROLLS - stone.rolls[1] + stone.lines[1] < GOOD
        || stone.lines[2] > BAD
    {
        return BigRational::new(BigInt::from(0u8), BigInt::from(1u8));
    }

    let max = (0..3)
        .filter(|line| stone.rolls[*line] < ROLLS)
        .map(|line| recurse_weighted(stone, cache, line))
        .max()
        .unwrap_or_else(|| BigRational::new(BigInt::from(0u8), BigInt::from(1u8)));

    cache.insert(stone, max.clone());
    max
}
