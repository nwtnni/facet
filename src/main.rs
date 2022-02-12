fn main() {
    let (line, value) = facet::evaluate(facet::Stone::default());
    println!("{}: {}", line, value);
}
