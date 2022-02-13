use std::collections::HashMap;
use std::fmt;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Stone {
    chance: Chance,
    lines: [u8; 3],
    rolls: [u8; 3],
}

impl Stone {
    pub fn new(chance: Chance, lines: [u8; 3], rolls: [u8; 3]) -> Self {
        Stone {
            chance,
            lines,
            rolls,
        }
    }

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

pub fn expectimax(stone: Stone, lines: [u8; 3], rolls: [u8; 3]) -> [u128; 3] {
    let mut expectimax = Expectimax::new(lines, rolls);
    let mut values = [0u128; 3];

    for line in 0..3 {
        values[line] = expectimax.select(stone, line);
    }

    values
}

struct Expectimax {
    cache: HashMap<Stone, u128>,
    lines: [u8; 3],
    rolls: [u8; 3],
}

impl Expectimax {
    fn new(lines: [u8; 3], rolls: [u8; 3]) -> Self {
        Expectimax {
            cache: HashMap::new(),
            lines,
            rolls,
        }
    }

    fn value(&self, stone: &Stone) -> Option<u128> {
        // Impossible to reach goal for any one line
        if stone.lines[0] + self.rolls[0] - stone.rolls[0] < self.lines[0]
            || stone.lines[1] + self.rolls[1] - stone.rolls[1] < self.lines[1]
            || stone.lines[2] > self.lines[2]
        {
            return Some(0);
        }

        // Successfully reached goal for all three lines
        if stone.rolls.into_iter().eq(self.rolls)
            && stone.lines[0] >= self.lines[0]
            && stone.lines[1] >= self.lines[1]
            && stone.lines[2] <= self.lines[2]
        {
            return Some(1);
        }

        None
    }

    fn expected(&mut self, stone: Stone) -> u128 {
        if let Some(value) = self
            .cache
            .get(&stone)
            .copied()
            .or_else(|| self.value(&stone))
        {
            return value;
        }

        let rolls = self.rolls;
        let max = (0..3)
            .filter(|line| stone.rolls[*line] < rolls[*line])
            .map(|line| self.select(stone, line))
            .max()
            .unwrap_or_default();

        self.cache.insert(stone, max);
        max
    }

    fn select(&mut self, stone: Stone, line: usize) -> u128 {
        let success = self
            .expected(stone.succeed(line))
            .checked_mul(stone.chance.success());

        let failure = self
            .expected(stone.fail(line))
            .checked_mul(stone.chance.failure());

        success
            .zip(failure)
            .and_then(|(success, failure)| success.checked_add(failure))
            .expect("[INTERNAL ERROR]: probability overflowed u128")
    }
}
