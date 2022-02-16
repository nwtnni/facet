use std::cmp;
use std::fmt;

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
    pub fn all() -> [Chance; 6] {
        [
            Chance::P25,
            Chance::P35,
            Chance::P45,
            Chance::P55,
            Chance::P65,
            Chance::P75,
        ]
    }

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
        numerators[line] = expectimax.select(&stone, line);
    }

    (numerators, denominator)
}

struct Expectimax {
    cache: Vec<U192>,
    lines: [u8; 3],
    rolls: [u8; 3],
    sizes: [usize; 3],
}

impl Expectimax {
    /// Construct a new cache for evaluation.
    fn new(lines: [u8; 3], rolls: [u8; 3]) -> Self {
        let mut sizes = [0; 3];

        (0..3)
            .map(|line| (rolls[line] + 1) * (rolls[line] + 2) / 2)
            .zip(&mut sizes)
            .for_each(|(_size, size)| *size = _size as usize);

        let cache = vec![U192::MAX; sizes.into_iter().product::<usize>() * Chance::all().len()];

        let mut expectimax = Expectimax {
            cache,
            lines,
            rolls,
            sizes,
        };

        expectimax.compute_terminal();
        expectimax.compute_all();
        expectimax
    }

    /// Mapping from state to unique index in global ordering.
    fn index(&self, stone: &Stone) -> usize {
        let mut index = stone.chance as usize;

        for line in 0..3 {
            let size = self.sizes[line];
            let roll = stone.rolls[line] as usize;
            let line = stone.lines[line] as usize;

            index *= size;
            index += roll * (roll + 1) / 2 + line;
        }

        index
    }

    fn compute_terminal(&mut self) {
        for chance in Chance::all() {
            for line_three in 0..self.rolls[2] + 1 {
                for line_two in 0..self.rolls[1] + 1 {
                    for line_one in 0..self.rolls[0] + 1 {
                        let stone =
                            Stone::new(chance, [line_one, line_two, line_three], self.rolls);

                        let index = self.index(&stone);

                        self.cache[index] = match stone.lines[0] >= self.lines[0]
                            && stone.lines[1] >= self.lines[1]
                            && stone.lines[2] <= self.lines[2]
                        {
                            true => U192::from(1u8),
                            false => U192::from(0u8),
                        };
                    }
                }
            }
        }
    }

    fn compute_all(&mut self) {
        for roll_total in (0..self.rolls.into_iter().sum::<u8>()).rev() {
            let min_three = roll_total
                .saturating_sub(self.rolls[1])
                .saturating_sub(self.rolls[0]);
            let max_three = cmp::min(roll_total, self.rolls[2]);

            for roll_three in min_three..=max_three {
                let min_two = roll_total
                    .saturating_sub(roll_three)
                    .saturating_sub(self.rolls[0]);
                let max_two = cmp::min(roll_total - roll_three, self.rolls[1]);

                for roll_two in min_two..=max_two {
                    let roll_one = roll_total - roll_three - roll_two;

                    for chance in Chance::all() {
                        for line_three in (0..roll_three + 1).rev() {
                            for line_two in (0..roll_two + 1).rev() {
                                for line_one in (0..roll_one + 1).rev() {
                                    let stone = Stone::new(
                                        chance,
                                        [line_one, line_two, line_three],
                                        [roll_one, roll_two, roll_three],
                                    );

                                    let index = self.index(&stone);

                                    if self.cache[index] < U192::MAX {
                                        continue;
                                    }

                                    self.cache[index] = (0..3)
                                        .map(|line| self.select(&stone, line))
                                        .max()
                                        .unwrap_or_default();
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Compute the expected value for `stone` if this line is selected.
    fn select(&mut self, stone: &Stone, line: usize) -> U192 {
        if stone.rolls[line] >= self.rolls[line] {
            return U192::from(0u8);
        }

        stone.chance.success() * self.cache[self.index(&stone.succeed(line))]
            + stone.chance.failure() * self.cache[self.index(&stone.fail(line))]
    }
}
