use std::num::ParseIntError;
use either::Either;
use pest::Parser;
use pest_derive::Parser;
use thiserror::Error;

#[derive(Parser)]
#[grammar = "idf30.pest"]
struct Idf30Parser;

#[derive(Error, Debug)]
pub enum Error {
    #[error("File does not contain header section or is empty")]
    MissingHeader,
    #[error("Expected version 3.0")]
    UnsupportedVersion,
    #[error("Expected BOARD_FILE or PANEL_FILE")]
    WrongFileType,
    #[error("MM or THOU expected")]
    WrongUnit,
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
    #[error(transparent)]
    Pest(#[from] pest::error::Error<Rule>),
    #[error("Internal grammar error")]
    GrammarExpectedPair,
    #[error("Expected different rule, got: {:?}", .0)]
    GrammarExpectedRule(Rule)
}

#[derive(Clone, Debug)]
pub struct Idf30<'a> {
    pub header: Header<'a>,
    pub placement: Vec<Component<'a>>,
    pub other_sections: Vec<IdfSection<'a>>
}

#[derive(Clone, Debug)]
pub struct Header<'a> {
    ty: FileType,
    source: Either<&'a str, String>,
    date: Either<&'a str, String>,
    board_file_version: u32,
    board_name: Either<&'a str, String>,
    units: Unit
}

#[derive(Clone, Debug)]
pub enum FileType {
    BoardFile,
    PanelFile
}

#[derive(Clone, Debug)]
pub enum Unit {
    SImm,
    Mils
}

#[derive(Clone, Debug)]
pub struct IdfSection<'a> {
    /// e.g. BOARD_OUTLINE
    name: Either<&'a str, String>,
    /// e.g. ECAD in 'BOARD_OUTLINE ECAD'
    args: Vec<Either<&'a str, String>>,
    records: Vec<IdfValue<'a>>
}

#[derive(Clone, Debug)]
pub struct Component<'a> {
    pub package_name: Either<&'a str, String>,
    pub part_number: Either<&'a str, String>,
    pub designator: ReferenceDesignator<'a>,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub board_side: BoardSide,
    pub placement_status: PlacementStatus,
}

#[derive(Clone, Debug)]
pub enum ReferenceDesignator<'a> {
    Any(Either<&'a str, String>),
    NoRefDes,
    Board
}

#[derive(Clone, Debug)]
pub enum BoardSide {
    Top,
    Bottom
}

#[derive(Clone, Debug)]
pub enum PlacementStatus {
    Placed,
    Unplaced,
    MCad,
    ECad
}

#[derive(Clone, PartialEq, Debug)]
pub enum IdfValue<'a> {
    Integer(i64),
    Float(f64),
    String(Either<&'a str, String>),
}

macro_rules! next_inner {
    ($pairs:expr) => {
        $pairs.next().ok_or(Error::GrammarExpectedPair)?.into_inner()
    };
}

macro_rules! next_str {
    ($pairs:expr) => {{
        let pair = $pairs.next().ok_or(Error::GrammarExpectedPair)?;
        if pair.as_rule() == Rule::string || pair.as_rule() == Rule::string_num_allowed || pair.as_rule() == Rule::quoted_string {
            pair.as_str()
        } else {
            return Err(Error::GrammarExpectedRule(pair.as_rule()));
        }
    }};
}

macro_rules! next_int {
    ($pairs:expr) => {

    };
}

macro_rules! next_float {
    ($pairs:expr) => {

    };
}

pub fn parse_idf30_file(file: &str) -> Result<Idf30, Error> {
    let mut idf30 = Idf30Parser::parse(Rule::idf30, file)?;
    // println!("{idf30:?}");
    let mut header_section = next_inner!(idf30);
    if next_str!(next_inner!(next_inner!(header_section))) != "HEADER" {
        return Err(Error::MissingHeader);
    }
    let mut header_record0 = next_inner!(header_section);
    let ty = match next_str!(header_record0) {
        "BOARD_FILE" => FileType::BoardFile,
        "PANEL_FILE" => FileType::PanelFile,
        _ => {
            return Err(Error::WrongFileType)
        }
    };
    if next_str!(header_record0) != "3.0" {
        return Err(Error::UnsupportedVersion)
    }
    let source = Either::Left(next_str!(header_record0));
    let date = Either::Left(next_str!(header_record0));
    let board_file_version = next_str!(header_record0).parse()?;
    let mut header_record1 = next_inner!(header_section);
    let board_name = Either::Left(next_str!(header_record1));
    let units = match next_str!(header_record1) {
        "MM" => Unit::SImm,
        "THOU" => Unit::Mils,
        _ => {
            return Err(Error::WrongUnit);
        }
    };
    Ok(Idf30 {
        header: Header {
            ty,
            source,
            date,
            board_file_version,
            board_name,
            units,
        },
        placement: vec![],
        other_sections: vec![]
    })
}
