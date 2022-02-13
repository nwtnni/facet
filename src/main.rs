use std::env;
use std::fs;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Facet {
    chance: u8,

    current_lines: [u8; 3],
    current_rolls: [u8; 3],

    target_lines: [u8; 3],
    target_rolls: [u8; 3],
}

fn main() {
    let facet = env::args()
        .nth(1)
        .map(fs::read_to_string)
        .expect("[ERROR]: expected usage: `./facet <PATH>`, where `<PATH>` is the path to a valid JSON file")
        .map(|command| serde_json::from_str::<Facet>(&command))
        .expect("[ERROR]: could not read file at <PATH>")
        .expect("[ERROR]: invalid JSON file");

    let chance = match facet.chance {
        25 => facet::Chance::P25,
        35 => facet::Chance::P35,
        45 => facet::Chance::P45,
        55 => facet::Chance::P55,
        65 => facet::Chance::P65,
        75 => facet::Chance::P75,
        _ => panic!(
            "Expected chance to be in [25, 35, ..., 75], but got: {}",
            facet.chance
        ),
    };

    let stone = facet::Stone::new(chance, facet.current_lines, facet.current_rolls);
    let (numerators, denominator) =
        facet::expectimax(stone, facet.target_lines, facet.target_rolls);

    for (line, numerator) in numerators.into_iter().enumerate() {
        const PRECISION: u32 = 4;

        let approximate = (numerator * facet::U192::from(10u64.pow(PRECISION)) / denominator)
            .as_u64() as f64
            / 10u64.pow(PRECISION) as f64;

        println!("Line {}: ({:.2$})", line, approximate, PRECISION as usize);
    }
}
