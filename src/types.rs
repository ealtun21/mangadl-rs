use std::fmt::{Display, Formatter};

use crossterm::style::Stylize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveType {
    Images,
    ImagesChapter,
    PdfSplit,
    PdfSingle,
    Urls,
}

pub enum Test1 {
    Images,
}

impl Display for SaveType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveType::Urls => write!(
                f,
                "{} {} {} {}",
                "Text".blue(),
                "Text file of URLS".white(),
                "(Fastest Save)".dark_green(),
                "(Lowest RAM Usage)".green()
            ),
            SaveType::PdfSplit => write!(
                f,
                "{} {} {} {}",
                "PDF".blue(),
                "Multiple pdfs split by chapters".white(),
                "(Slow Save)".dark_yellow(),
                "(High RAM Usage)".red()
            ),
            SaveType::PdfSingle => write!(
                f,
                "{} {} {} {}",
                "PDF".blue(),
                "Single pdf".white(),
                "(Slowest Save)".dark_red(),
                "(High RAM Usage)".red()
            ),
            SaveType::ImagesChapter => write!(
                f,
                "{} {} {} {}",
                "Images".blue(),
                "Multiple folders split by chapters containing images".white(),
                "(Fast Save)".green(),
                "(Low RAM Usage)".green()
            ),
            SaveType::Images => write!(
                f,
                "{} {} {} {}",
                "Images".blue(),
                "Folder containing images".white(),
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
