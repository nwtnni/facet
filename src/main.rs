fn main() {
    let (_, probability) = facet::evaluate(facet::Stone::default());

    let numerator = probability / 10u128.pow(26);
    let denominator = 2u128.pow(30) * 10u128.pow(4);

    println!("{}", numerator as f64 / denominator as f64);
}
