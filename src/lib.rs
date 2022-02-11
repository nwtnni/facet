use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;

use num_bigint::BigInt;
use num_rational::BigRational;
use priority_queue::PriorityQueue;

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

pub fn search() {
    let mut queue = PriorityQueue::new();
    let mut known = HashSet::new();
    let mut track = HashMap::new();

    queue.push(
        Stone::default(),
        BigRational::new(BigInt::from(1u8), BigInt::from(1u8)),
    );

    while let Some((stone, probability)) = queue.pop() {
        if !known.insert(stone) {
            continue;
        }

        if stone.lines[0] >= 7 && stone.lines[1] >= 7 && stone.lines[2] <= 4 {
            let mut stones = vec![stone];
            let mut next = stone;

            while let Some(prev) = track.get(&next).copied() {
                stones.push(prev);
                next = prev;
            }

            stones.reverse();

            for stone in stones {
                println!("{}", stone);
            }

            return;
        }

        for line in (0..3).filter(|line| stone.rolls[*line] < 10) {
            let success = stone.succeed(line);
            let success_probabilty = stone.chance.to_probability_success() * &probability;

            if match queue.get_priority(&success) {
                None => true,
                Some(probability) => success_probabilty > *probability,
            } {
                queue.push(success, success_probabilty);
                track.insert(success, stone);
            }

            let failure = stone.fail(line);
            let failure_probability = stone.chance.to_probability_failure() * &probability;

            if match queue.get_priority(&failure) {
                None => true,
                Some(probability) => failure_probability > *probability,
            } {
                queue.push(failure, failure_probability);
                track.insert(failure, stone);
            }
        }
    }
}
