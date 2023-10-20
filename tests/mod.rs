#[cfg(test)]
mod tests {
    use idf::idf30::{FileType, Idf30};

    #[test]
    fn board_file_parse() {
        let contents = std::fs::read_to_string("./tests/board.idf").unwrap();
        let board = Idf30::parse(&contents).unwrap();
        assert!(matches!(board.header.ty, FileType::BoardFile { .. }));
    }

    #[test]
    fn library_file_parse() {
        let contents = std::fs::read_to_string("./tests/library.idf").unwrap();
        let lib = Idf30::parse(&contents).unwrap();
        assert!(matches!(lib.header.ty, FileType::LibraryFile { .. }));
        println!("{lib:#?}");
    }
}