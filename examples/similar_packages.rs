use std::env;
use std::fmt::format;
use std::fs::read_to_string;
use either::Either;
use idf::idf30::{FileType, Idf30};

fn main() {
    let mut args = env::args().into_iter().skip(1);
    let idf_path = args.next().expect("IDF file path");
    let ldf_path = args.next().expect("LDF file path");

    let idf_contents = read_to_string(idf_path).unwrap();
    let mut idf_file = Idf30::parse(idf_contents.as_str()).unwrap();
    idf_file.header.source = Either::Right(format!("rust_idf_{}", idf_file.header.source));

    let ldf_contents = read_to_string(ldf_path).unwrap();
    let mut ldf_file = Idf30::parse(ldf_contents.as_str()).unwrap();
    ldf_file.header.source = Either::Right(format!("rust_idf_{}", idf_file.header.source));

    if let FileType::BoardFile { board_name, .. } = &idf_file.header.ty {
        println!("Name: {}\nComponents placed: {}", board_name, idf_file.placement.len());

        let mut removed = 0;
        idf_file.placement.retain(|c| {
            if c.designator.is_test_point() {
                removed += 1;
                false
            } else {
                true
            }
        });
        for component in &mut idf_file.placement {
            component.part_number = component.package_name.clone();
        }
        println!("Removed: {removed}");
    }

    if let FileType::LibraryFile { components } = &mut ldf_file.header.ty {
        println!("Components defs: {}", components.len());
        for def in components.iter_mut() {
            def.part_number = def.geometry_name.clone();
        }
        let mut seen = vec![];
        components.retain(|c| {
            let name = format!("{}", c.geometry_name);
            if seen.contains(&name) {
                false
            } else {
                seen.push(name);
                true
            }
        });
        println!("After removing duplicates: {}", seen.len());
    }

    std::fs::write("./out.idf", idf_file.to_string()).expect("Write file failed");
    std::fs::write("./out.ldf", ldf_file.to_string()).expect("Write file failed");
}
