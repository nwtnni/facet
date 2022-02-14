use std::fmt;

use rustc_hash::FxHashMap;
use wasm_bindgen::prelude::wasm_bindgen;

uint::construct_uint! {
    pub struct U192(3);
}

/// Represents the current state of an ability stone being faceted.
#[wasm_bindgen]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Stone {
    /// Current chance of success.
    chance: Chance,

    /// Number of successful rolls per line.
    lines: [u8; 3],

    /// Number of total attempts per line.
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

    /// State transition after succeeding a roll for `line`.
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

    /// State transition after failing a roll for `line`.
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

#[wasm_bindgen]
impl Stone {
    // The WebAssembly ABI doesn't accept fixed-size arrays, so this is a
    // workaround that avoids allocation and bounds checking.
    #[wasm_bindgen(constructor)]
    pub fn new_wasm(
        chance: Chance,
        line_0: u8,
        line_1: u8,
        line_2: u8,
        roll_0: u8,
        roll_1: u8,
        roll_2: u8,
    ) -> Stone {
        Stone {
            chance,
            lines: [line_0, line_1, line_2],
            rolls: [roll_0, roll_1, roll_2],
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

/// Represents the set of valid success rates during faceting.
#[wasm_bindgen]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Chance {
    P25 = 0,
    P35 = 1,
    P45 = 2,
    P55 = 3,
    P65 = 4,
    P75 = 5,
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
    /// State transition after succeeding a roll.
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

    /// State transition after failing a roll.
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

    /// Probability of success, mulitplied by 20.
    pub fn success(&self) -> U192 {
        U192::from(match self {
            Chance::P25 => 5,
            Chance::P35 => 7,
            Chance::P45 => 9,
            Chance::P55 => 11,
            Chance::P65 => 13,
            Chance::P75 => 15,
        })
    }

    /// Probability of failure, multiplied by 20.
    pub fn failure(&self) -> U192 {
        U192::from(match self {
            Chance::P25 => 15,
            Chance::P35 => 13,
            Chance::P45 => 11,
            Chance::P55 => 9,
            Chance::P65 => 7,
            Chance::P75 => 5,
        })
    }
}

#[wasm_bindgen]
pub fn expectimax_wasm(
    stone: Stone,
    line_0: u8,
    line_1: u8,
    line_2: u8,
    roll_0: u8,
    roll_1: u8,
    roll_2: u8,
    precision: u32,
) -> Box<[f64]> {
    let (numerators, denominator) =
        expectimax(stone, [line_0, line_1, line_2], [roll_0, roll_1, roll_2]);

    let inflate = U192::from(10u64.pow(precision));
    let deflate = 10u64.pow(precision) as f64;

    numerators
        .into_iter()
        .map(|numerator| (numerator * inflate / denominator).as_u64() as f64 / deflate)
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

/// Compute the probability of reaching target `lines`, given limit `rolls`,
/// current state `stone`, and optimal decision-making.
///
/// Returns the numerators for each choice, and the denominator for all three.
pub fn expectimax(stone: Stone, lines: [u8; 3], rolls: [u8; 3]) -> ([U192; 3], U192) {
    let mut expectimax = Expectimax::new(lines, rolls);
    let mut numerators = [U192::from(0u8); 3];

    // The denominator is given by the 20 ^ (recursion depth), and the
    // recursion depth is (# rolls available) - (# rolls used).
    let denominator = U192::from(20u8).pow(U192::from(
        rolls.into_iter().sum::<u8>() - stone.rolls.into_iter().sum::<u8>(),
    ));

    for line in 0..3 {
        numerators[line] = expectimax.select(stone, line);
    }

    (numerators, denominator)
}

struct Expectimax {
    cache: FxHashMap<Stone, U192>,
    lines: [u8; 3],
    rolls: [u8; 3],
}

impl Expectimax {
    /// Construct a new cache for evaluation.
    fn new(lines: [u8; 3], rolls: [u8; 3]) -> Self {
        Expectimax {
            cache: FxHashMap::default(),
            lines,
            rolls,
        }
    }

    /// Compute the value of `stone`, if it is terminal.
    fn value(&self, stone: &Stone) -> Option<U192> {
        // Impossible to reach goal for any one line
        if stone.lines[0] + self.rolls[0] - stone.rolls[0] < self.lines[0]
            || stone.lines[1] + self.rolls[1] - stone.rolls[1] < self.lines[1]
            || stone.lines[2] > self.lines[2]
        {
            return Some(U192::from(0));
        }

        // Successfully reached goal for all three lines
        //
        // Note: it's important that we don't short-circuit evaluation
        // here at a shallower recursion depth. Because we don't explicitly
        // calculate denominators, this `1` is only meaningful if all
        // of them are summed at the deepest recursion depth.
        if stone.rolls.into_iter().eq(self.rolls)
            && stone.lines[0] >= self.lines[0]
            && stone.lines[1] >= self.lines[1]
            && stone.lines[2] <= self.lines[2]
        {
            return Some(U192::from(1));
        }

        None
    }

    /// Compute the maximum value for `stone`.
    fn expected(&mut self, stone: Stone) -> U192 {
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

    /// Compute the expected value for `stone` if this line is selected.
    fn select(&mut self, stone: Stone, line: usize) -> U192 {
        let success = self
            .expected(stone.succeed(line))
            .checked_mul(stone.chance.success());

        let failure = self
            .expected(stone.fail(line))
            .checked_mul(stone.chance.failure());

        success
            .zip(failure)
            .and_then(|(success, failure)| success.checked_add(failure))
            .expect("[INTERNAL ERROR]: probability overflowed U192")
    }
}
