use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveType {
    Urls,
    Images,
}

impl Display for SaveType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveType::Urls => write!(f, "List of URLS"),
            SaveType::Images => write!(f, "Images inside Folders"),
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
