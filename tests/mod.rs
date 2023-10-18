#[cfg(test)]
mod tests {
    use std::fs::File;
    use idf::idf30::parse_idf30_file;

    #[test]
    fn simple_file_parse() {
        let contents = std::fs::read_to_string("./tests/simple.idf").unwrap();
        let idf30 = parse_idf30_file(&contents).unwrap();
        println!("{idf30:?}");
    }
}