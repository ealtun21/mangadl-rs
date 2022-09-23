use image::DynamicImage;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{Display, Formatter};
use tokio::task::JoinError;

use crate::{chapter::Chapter, fetch::get_img};

const URL: &str = "https://mangasee123.com/";
const COVER_URL: &str = "https://temp.compsci88.com/";

// Names taken directly from mangasee123, rename was deemed unnecessary.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Manga {
    pub i: String,
    pub s: String,
    pub o: String,
    pub ss: String,
    pub ps: String,
    pub t: String,
    pub v: String,
    pub vm: String,
    pub y: String,
    pub a: Vec<String>,
    pub al: Vec<String>,
    pub l: String,
    pub lt: i128,
    pub ls: Value,
    pub g: Vec<String>,
    pub h: bool,
}

impl Manga {
    pub async fn urls_all(&self) -> Vec<String> {
        let mut urls: Vec<String> = Vec::new();
        let chapters = Chapter::list(self.i.as_str()).await.unwrap();

        let mut one_chapters_urls = self.chapters_urls_multi(chapters).await;
        urls.append(&mut one_chapters_urls);
        urls
    }

    pub async fn chapters_urls_multi(&self, chapters: Vec<Chapter>) -> Vec<String> {
        let mut urls = Vec::new();

        // Split chapters into 16 chunks
        let chunks = chapters.chunks(if chapters.len() / 16 == 0 {
            1
        } else {
            chapters.len() / 16
        });

        // Spawn a tread for each chunk
        let mut handles = Vec::new();
        for chunk in chunks {
            let myself = self.clone();
            let chunk = chunk.to_vec();
            let handle =
                tokio::spawn(async move { Self::chapters_urls_single(&myself, chunk).await });
            handles.push(handle);
        }

        // Join all the handles
        for handle in handles {
            let mut one_chapters_urls = handle.await.unwrap();
            urls.append(&mut one_chapters_urls);
        }
        urls
    }

    pub async fn chapters_urls_single(&self, chapters: Vec<Chapter>) -> Vec<String> {
        let mut urls = Vec::new();

        for chapter in chapters {
            let url = chapter.cur_path_name(self.i.as_str()).await.unwrap();
            for page in 1..chapter.Page.parse::<usize>().unwrap() {
                urls.push(format!(
                    "https://{}/manga/{}{}/{:0>4}-{:0>3}.png",
                    url,
                    self.i,
                    chapter.directory(),
                    chapter.to_url_id(),
                    page
                ));
            }
        }
        urls
    }

    pub async fn all_manga_list() -> Result<Vec<Manga>, Box<dyn std::error::Error>> {
        let page = reqwest::get(format!("{URL}search/")).await?.text().await?;
        Ok(serde_json::from_str(
            Regex::new(r#"vm\.Directory = (.*);"#)
                .expect("Failed to create regex")
                .captures(&page)
                .expect("Failed to find manga list")
                .get(1)
                .expect("Failed to get manga list")
                .as_str(),
        )?)
    }

    pub async fn cover(&self) -> Result<DynamicImage, reqwest::Error> {
        get_img(format!("{COVER_URL}cover/{}.jpg", self.i).as_str()).await
    }

    pub async fn covers_all(
        manga_list: Vec<Manga>,
    ) -> Result<Vec<Result<DynamicImage, reqwest::Error>>, JoinError> {
        let mut covers = Vec::new();
        let mut handles = Vec::new();
        for manga in manga_list {
            let handle = tokio::spawn(async move { manga.cover().await });
            handles.push(handle);
        }
        for handle in handles {
            covers.push(handle.await?);
        }
        Ok(covers)
    }

    #[must_use]
    pub fn filter_manga(genres: Vec<String>, manga: Vec<Manga>) -> Option<Vec<Manga>> {
        let mut filtered_manga = manga;
        for genre in genres {
            filtered_manga.retain(|manga| manga.g.contains(&genre));
        }
        Some(filtered_manga)
    }

    pub async fn find_all_genre(manga: &Vec<Manga>) -> Vec<String> {
        let mut genres: Vec<String> = Vec::new();
        for m in manga {
            for g in &m.g {
                if !genres.contains(g) {
                    genres.push(g.to_string());
                }
            }
        }
        genres
    }
}

impl Display for Manga {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.al.is_empty() {
            write!(f, "{}", self.s)
        } else {
            write!(f, "{} ({})", self.s, self.al.join(", "))
        }
    }
}
