use std::fmt::{Display, Formatter};
use std::num::{ParseFloatError, ParseIntError};
use either::Either;
use pest::iterators::Pairs;
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
    #[error("Expected 2 records per component, got 1")]
    MalformedPlacementSection,
    #[error("{}", .0)]
    Malformed(&'static str),
    #[error(transparent)]
    ParseInt(#[from] ParseIntError),
    #[error(transparent)]
    ParseFloat(#[from] ParseFloatError),
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

impl<'a> Display for Header<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            ".HEADER\n{} 3.0 {} {} {}\n{} {}\n.END_HEADER\n",
            self.ty,
            self.source,
            self.date,
            self.board_file_version,
            self.board_name,
            self.units
        )
    }
}

#[derive(Clone, Debug)]
pub enum FileType {
    BoardFile,
    PanelFile
}

impl Display for FileType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FileType::BoardFile => write!(f, "BOARD_FILE"),
            FileType::PanelFile => write!(f, "PANEL_FILE")
        }
    }
}

#[derive(Clone, Debug)]
pub enum Unit {
    SImm,
    Mils
}

impl Display for Unit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Unit::SImm => write!(f, "MM"),
            Unit::Mils => write!(f, "THOU"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct IdfSection<'a> {
    /// e.g. BOARD_OUTLINE
    name: Either<&'a str, String>,
    /// e.g. ECAD in 'BOARD_OUTLINE ECAD'
    args: Vec<Either<&'a str, String>>,
    records: Vec<Vec<IdfValue<'a>>>
}

impl<'a> Display for IdfSection<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let args: String = self.args.iter().map(|arg| format!(" {arg}")).collect();
        let mut records = String::new();
        for record in self.records.iter() {
            records.push_str(" ");
            for v in record {
                records.push_str(format!(" {v}").as_str());
            }
            records.push_str("\n");
        }
        write!(f, ".{}{}\n{}.END_{}\n", self.name, args, records, self.name)
    }
}

#[derive(Clone, Debug)]
pub struct Component<'a> {
    pub package_name: Either<&'a str, String>,
    pub part_number: Either<&'a str, String>,
    pub designator: ReferenceDesignator<'a>,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rotation: f32,
    pub board_side: BoardSide,
    pub placement_status: PlacementStatus,
}

impl<'a> Display for Component<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}\n  {:.4} {:.4} {:.4} {:.3} {} {}\n",
            escape_string(&self.package_name),
            escape_string(&self.part_number),
            self.designator,
            self.x, self.y, self.z, self.rotation,
            self.board_side, self.placement_status
        )
    }
}

#[derive(Clone, Debug)]
pub enum ReferenceDesignator<'a> {
    Any(Either<&'a str, String>),
    NoRefDes,
    Board
}

impl<'a> Display for ReferenceDesignator<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ReferenceDesignator::Any(d) => write!(f, "{d}"),
            ReferenceDesignator::NoRefDes => write!(f, "NOREFDES"),
            ReferenceDesignator::Board => write!(f, "BOARD")
        }
    }
}

#[derive(Clone, Debug)]
pub enum BoardSide {
    Top,
    Bottom
}

impl Display for BoardSide {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BoardSide::Top => write!(f, "TOP"),
            BoardSide::Bottom => write!(f, "BOTTOM")
        }
    }
}

#[derive(Clone, Debug)]
pub enum PlacementStatus {
    Placed,
    Unplaced,
    MCad,
    ECad
}

impl Display for PlacementStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PlacementStatus::Placed => write!(f, "PLACED"),
            PlacementStatus::Unplaced => write!(f, "UNPLACED"),
            PlacementStatus::MCad => write!(f, "MCAD"),
            PlacementStatus::ECad => write!(f, "ECAD"),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum IdfValue<'a> {
    Integer(i64),
    Float(f64),
    String(Either<&'a str, String>),
}

impl<'a> Display for IdfValue<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            IdfValue::Integer(x) => write!(f, "{x}"),
            IdfValue::Float(x) => write!(f, "{x:.4}"),
            IdfValue::String(s) => write!(f, "{}", escape_string(s))
        }
    }
}

fn escape_string<'a: 'b, 'b>(s: &'b Either<&'a str, String>) -> &'b str {
    match s {
        Either::Left(s) => if s.is_empty() {
            "\"\""
        } else {
            s
        }
        Either::Right(s) => if s.is_empty() {
            "\"\""
        } else {
            s.as_str()
        }
    }
}

macro_rules! next_inner {
    ($pairs:expr) => {
        $pairs.next().ok_or(Error::GrammarExpectedPair)?.into_inner()
    };
}

macro_rules! next_str {
    ($pairs:expr) => {{
        let pair = $pairs.next().ok_or(Error::GrammarExpectedPair)?;
        if pair.as_rule() == Rule::string || pair.as_rule() == Rule::string_num_allowed {
            pair.as_str()
        } else if pair.as_rule() == Rule::quoted_string {
            pair.into_inner().as_str()
        } else {
            return Err(Error::GrammarExpectedRule(pair.as_rule()));
        }
    }};
}

// macro_rules! next_int {
//     ($pairs:expr) => {
//
//     };
// }

macro_rules! next_float {
    ($pairs:expr) => {{
        let pair = $pairs.next().ok_or(Error::GrammarExpectedPair)?;
        if pair.as_rule() == Rule::float {
            pair.as_str().parse()?
        } else {
            return Err(Error::GrammarExpectedRule(pair.as_rule()));
        }
    }};
}

impl<'a> Idf30<'a> {
    pub fn parse(file: &str) -> Result<Idf30, Error> {
        let mut idf30 = Idf30Parser::parse(Rule::idf30, file)?;
        // println!("{idf30:#?}");
        let header = parse_header(&mut idf30)?;
        let mut placement = vec![];
        let mut other_sections = vec![];
        while let Some(section) = idf30.next() {
            if section.as_rule() == Rule::EOI {
                break;
            }
            let mut section = section.into_inner();
            let mut section_header = next_inner!(section);
            let section_name = next_str!(next_inner!(section_header));
            if section_name == "PLACEMENT" {
                while let Some(record) = section.next() {
                    if record.as_rule() == Rule::section_name {
                        break;
                    }
                    let record = record.into_inner();
                    let component = parse_component(&mut section, record)?;
                    placement.push(component);
                }
            } else {
                let args = section_header.into_iter().map(|arg| Either::Left(arg.as_str())).collect();
                let mut records = vec![];
                while let Some(record) = section.next() {
                    if record.as_rule() == Rule::section_name {
                        break;
                    }
                    let record = record.into_inner();
                    let values: Result<Vec<IdfValue>, Error> = record.into_iter().map(|p| {
                        match p.as_rule() {
                            Rule::integer => {
                                Ok(IdfValue::Integer(p.as_str().parse()?))
                            }
                            Rule::float => {
                                Ok(IdfValue::Float(p.as_str().parse()?))
                            }
                            Rule::string => {
                                Ok(IdfValue::String(Either::Left(p.as_str())))
                            }
                            Rule::quoted_string => {
                                Ok(IdfValue::String(Either::Left(p.as_str())))
                            }
                            _ => {
                                return Err(Error::GrammarExpectedRule(Rule::value))
                            }
                        }
                    }).collect();
                    records.push(values?);
                }
                let section = IdfSection {
                    name: Either::Left(section_name),
                    args,
                    records,
                };
                other_sections.push(section);
            }
        }

        Ok(Idf30 {
            header,
            placement,
            other_sections,
        })
    }

    pub fn to_string(&self) -> String {
        let mut s = format!("{}", self.header);
        for o in &self.other_sections {
            s.push_str(format!("{o}").as_str())
        }
        s.push_str(".PLACEMENT\n");
        for c in &self.placement {
            s.push_str(format!("{c}").as_str())
        }
        s.push_str(".END_PLACEMENT\n");
        s
    }
}

fn parse_component<'a>(section: &mut Pairs<Rule>, mut record: Pairs<'a, Rule>) -> Result<Component<'a>, Error> {
    let package_name = Either::Left(next_str!(record));
    let part_number = Either::Left(next_str!(record));
    let designator = next_str!(record);
    let designator = match designator {
        "NOREFDES" => ReferenceDesignator::NoRefDes,
        "BOARD" => ReferenceDesignator::Board,
        d => ReferenceDesignator::Any(Either::Left(d))
    };
    let mut record = section.next().ok_or(Error::MalformedPlacementSection)?.into_inner();
    let x = next_float!(record);
    let y = next_float!(record);
    let z = next_float!(record);
    let rotation = next_float!(record);
    let side = next_str!(record);
    let board_side = match side {
        "TOP" => BoardSide::Top,
        "BOTTOM" => BoardSide::Bottom,
        _ => {
            return Err(Error::Malformed("Expected TOP or BOTTOM for side of board"));
        }
    };
    let placement_status = next_str!(record);
    let placement_status = match placement_status {
        "PLACED" => PlacementStatus::Placed,
        "UNPLACED" => PlacementStatus::Unplaced,
        "MCAD" => PlacementStatus::MCad,
        "ECAD" => PlacementStatus::ECad,
        _ => {
            return Err(Error::Malformed("Wrong placement status"));
        }
    };
    Ok(Component {
        package_name,
        part_number,
        designator,
        x,
        y,
        z,
        rotation,
        board_side,
        placement_status,
    })
}

fn parse_header<'a>(idf30: &mut Pairs<'a, Rule>) -> Result<Header<'a>, Error> {
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
    let header = Header {
        ty,
        source,
        date,
        board_file_version,
        board_name,
        units
    };
    Ok(header)
}
