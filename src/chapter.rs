#![allow(non_snake_case)]

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fmt::{Display, Formatter},
    time::Duration,
};
use tokio::time::sleep;

const URL: &str = "https://mangasee123.com/";

// Names taken directly from mangasee123, so they are not snake case. Rename was deemed unnecessary.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChapterInfo {
    pub Chapter: String,
    pub Type: String,
    pub Date: Value,
}

impl ChapterInfo {
    pub async fn list(manga_id: &str) -> Result<Vec<ChapterInfo>, Box<dyn std::error::Error>> {
        let page = reqwest::get(format!("{URL}manga/{manga_id}"))
            .await?
            .text()
            .await?;
        Ok(serde_json::from_str(
            Regex::new(r#"vm\.Chapters = (.*);"#)
                .expect("Failed to create regex")
                .captures(&page)
                .expect("Failed to find chapter list")
                .get(1)
                .expect("Failed to get chapter list")
                .as_str(),
        )?)
    }

    pub fn to_url_id(&self) -> String {
        let chapter = self.Chapter[1..self.Chapter.len() - 1].to_string();
        let odd = self.Chapter[self.Chapter.len() - 1..].to_string();
        if odd == "0" {
            chapter
        } else {
            format!("{}.{}", chapter, odd)
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Chapter {
    pub Chapter: String,
    pub Page: String,
    pub Directory: String,
}

impl Chapter {
    pub async fn list(
        manga_id: &str,
        valid_chapter: &str,
    ) -> Result<Vec<Chapter>, Box<dyn std::error::Error>> {
        let chapter = valid_chapter[1..valid_chapter.len() - 1].to_string();
        let odd = valid_chapter[valid_chapter.len() - 1..].to_string();

        let valid_chapter = if odd == "0" {
            chapter
        } else {
            format!("{}.{}", chapter, odd)
        };

        let page = reqwest::get(format!(
            "{URL}read-online/{manga_id}-chapter-{valid_chapter}.html"
        ))
        .await?
        .text()
        .await?;
        Ok(serde_json::from_str(
            Regex::new(r#"vm\.CHAPTERS = (.*);"#)
                .expect("Failed to create regex")
                .captures(&page)
                .expect("Failed to find chapter list")
                .get(1)
                .expect("Failed to get chapter list")
                .as_str(),
        )?)
    }

    pub async fn cur_path_name(&self, manga: &str) -> String {
        let chapter = self.clone();
        let page = loop {
            match loop {
                match reqwest::get(format!("{URL}read-online/{manga}-chapter-{chapter}.html")).await
                {
                    Ok(data) => break data,
                    Err(e) => {
                        eprintln!("Error: {e}, Retrying!");
                        sleep(Duration::from_millis(50)).await;
                        continue;
                    }
                }
            }
            .text()
            .await
            {
                Ok(data) => break data,
                Err(e) => {
                    eprintln!("Error: {e}, Retrying!");
                    sleep(Duration::from_millis(50)).await;
                    continue;
                }
            }
        };

        Regex::new(r#"vm\.CurPathName = (.*);"#)
            .expect("Failed to create regex")
            .captures(&page)
            .expect("Failed to find chapter list")
            .get(1)
            .expect("Failed to get chapter list")
            .as_str()
            .trim()
            .replace('\"', "")
    }

    #[must_use]
    pub fn to_url_id(&self) -> String {
        let chapter = self.Chapter[1..self.Chapter.len() - 1].to_string();
        let odd = self.Chapter[self.Chapter.len() - 1..].to_string();
        if odd == "0" {
            chapter
        } else {
            format!("{}.{}", chapter, odd)
        }
    }

    #[must_use]
    pub fn directory(&self) -> String {
        if self.Directory.is_empty() {
            String::new()
        } else {
            format!("/{}", self.Directory)
        }
    }
}

impl Display for Chapter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_url_id())
    }
}
