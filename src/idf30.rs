use pest::Parser;
use pest_derive::Parser;
use pest::error::Error;

#[derive(Parser)]
#[grammar = "idf30.pest"]
struct Idf30Parser;

#[derive(Clone, Debug)]
pub struct Idf30<'a> {
    pub sections: Vec<IdfSection<'a>>
}

#[derive(Clone, Debug)]
pub struct IdfSection<'a> {
    /// e.g. BOARD_OUTLINE
    name: &'a str,
    /// e.g. ECAD in 'BOARD_OUTLINE ECAD'
    args: Vec<&'a str>,
    records: Vec<IdfValue<'a>>
}

#[derive(Clone, PartialEq, Debug)]
pub enum IdfValue<'a> {
    Integer(i64),
    Float(f64),
    String(&'a str),
}

pub fn parse_idf30_file(file: &str) -> Result<Idf30, Error<Rule>> {
    let mut idf30 = Idf30Parser::parse(Rule::idf30, file)?;
    println!("{idf30:#?}");
    let header_section = idf30.next().unwrap();
    Ok(Idf30 {
        sections: vec![]
    })
}