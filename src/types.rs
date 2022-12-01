use std::{
    error::Error,
    fmt::{Display, Formatter},
    str::FromStr,
};

use crossterm::style::Stylize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveType {
    Images,
    ImagesChapter,
    PdfSplit,
    PdfSingle,
    Urls,
}

impl Display for SaveType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveType::Urls => write!(
                f,
                "{} {} {} {}",
                "Text".blue(),
                "Text file of URLS",
                "(Fastest Save)".dark_green(),
                "(Lowest RAM Usage)".green()
            ),
            SaveType::PdfSplit => write!(
                f,
                "{} {} {} {}",
                "PDF".blue(),
                "Multiple pdfs split by chapters",
                "(Slow Save)".dark_yellow(),
                "(High RAM Usage)".red()
            ),
            SaveType::PdfSingle => write!(
                f,
                "{} {} {} {}",
                "PDF".blue(),
                "Single pdf",
                "(Slowest Save)".dark_red(),
                "(High RAM Usage)".red()
            ),
            SaveType::ImagesChapter => write!(
                f,
                "{} {} {} {}",
                "Images".blue(),
                "Multiple folders split by chapters containing images",
                "(Fast Save)".green(),
                "(Low RAM Usage)".green()
            ),
            SaveType::Images => write!(
                f,
                "{} {} {} {}",
                "Images".blue(),
                "Folder containing images",
                "(Fast Save)".green(),
                "(Low RAM Usage)".green()
            ),
        }
    }
}

pub enum DownloadType {
    Single,
    Multi,
}

impl Display for DownloadType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadType::Single => write!(f, "Single-Threaded"),
            DownloadType::Multi => write!(f, "Multi-Threaded"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Thread {
    amount: u8,
}

impl Thread {
    pub fn new(amount: u8) -> Result<Self, Box<dyn Error>> {
        if amount > 0 {
            Ok(Self { amount })
        } else {
            Err("Amount of threads must be larger then 0".into())
        }
    }

    pub fn get(&self) -> u8 {
        self.amount
    }
}

impl Display for Thread {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Thread { amount } = self;
        write!(f, "{}", amount)
    }
}

impl FromStr for Thread {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let amount = s.parse::<u8>()?;
        Self::new(amount)
    }
}
