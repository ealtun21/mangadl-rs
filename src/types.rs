use std::fmt::{Display, Formatter};

use enum_iterator::Sequence;

#[derive(Debug, Clone, PartialEq, Eq, Sequence)]
pub enum SaveType {
    Images,
    PdfSplit,
    PdfSingle,
    Urls,
}

impl Display for SaveType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveType::Urls => write!(f, "Text file of URLS (Fastest Save) (Lowest RAM Usage)"),
            SaveType::PdfSplit => write!(f, "Multiple Pdfs split by Chapters (Slow Save) (High RAM Usage)"),
            SaveType::PdfSingle => write!(f, "Single Pdf (Slowest Save) (High RAM Usage)"),
            SaveType::Images => write!(f, "Folder containing images (Fast Save) (Low RAM Usage)"),
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
