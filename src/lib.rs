use std::collections::HashMap;
use std::fmt;

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

    pub fn success(&self) -> u128 {
        match self {
            Chance::P25 => 5,
            Chance::P35 => 7,
            Chance::P45 => 9,
            Chance::P55 => 11,
            Chance::P65 => 13,
            Chance::P75 => 15,
        }
    }

    pub fn failure(&self) -> u128 {
        match self {
            Chance::P25 => 15,
            Chance::P35 => 13,
            Chance::P45 => 11,
            Chance::P55 => 9,
            Chance::P65 => 7,
            Chance::P75 => 5,
        }
    }
}

const ROLLS: u8 = 10;
const GOOD: u8 = 7;
const BAD: u8 = 4;

pub fn expectimax(stone: Stone) -> (usize, u128) {
    let mut max_line = 0;
    let mut max_value = u128::from(0u8);
    let mut cache = HashMap::new();

    for line in 0..3 {
        let value = select(stone, &mut cache, line);
        if value > max_value {
            max_line = line;
            max_value = value;
        }
    }

    (max_line, max_value)
}

fn expected(stone: Stone, cache: &mut HashMap<Stone, u128>) -> u128 {
    if let Some(value) = cache.get(&stone) {
        return *value;
    }

    // Terminal state: success
    if stone.rolls.into_iter().all(|roll| roll == ROLLS)
        && stone.lines[0] >= GOOD
        && stone.lines[1] >= GOOD
        && stone.lines[2] <= BAD
    {
        return 1;
    }

    // Terminal state: failure
    if ROLLS - stone.rolls[0] + stone.lines[0] < GOOD
        || ROLLS - stone.rolls[1] + stone.lines[1] < GOOD
        || stone.lines[2] > BAD
    {
        return 0;
    }

    let max = (0..3)
        .filter(|line| stone.rolls[*line] < ROLLS)
        .map(|line| select(stone, cache, line))
        .max()
        .unwrap_or_default();

    cache.insert(stone, max);
    max
}

fn select(stone: Stone, cache: &mut HashMap<Stone, u128>, line: usize) -> u128 {
    let success = expected(stone.succeed(line), cache).checked_mul(stone.chance.success());
    let failure = expected(stone.fail(line), cache).checked_mul(stone.chance.failure());
    success
        .zip(failure)
        .and_then(|(success, failure)| success.checked_add(failure))
        .expect("[INTERNAL ERROR]: probability overflowed u128")
}
