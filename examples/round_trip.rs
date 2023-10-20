use either::Either;
use idf::idf30::Idf30;
use std::env;

fn main() {
    let path = env::args()
        .into_iter()
        .skip(1)
        .next()
        .expect("IDF file path");
    let contents = std::fs::read_to_string(path).unwrap();

    let mut file = Idf30::parse(&contents).unwrap();
    file.header.source = Either::Right(format!("rust_idf_{}", file.header.source));

    println!(
        "Name: {}\nComponents: {}",
        file.header.board_name,
        file.placement.len()
    );

    std::fs::write("./out.idf", file.to_string()).expect("Write file failed");
}
