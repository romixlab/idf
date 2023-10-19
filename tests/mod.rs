#[cfg(test)]
mod tests {
    use idf::idf30::Idf30;

    #[test]
    fn simple_file_parse() {
        let contents = std::fs::read_to_string("./tests/simple.idf").unwrap();
        let idf30 = Idf30::parse(&contents).unwrap();
        // println!("{idf30:#?}");
        println!("{}", idf30.to_string());
    }
}