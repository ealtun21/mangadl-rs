use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::slice::ParallelSliceMut;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{Display, Formatter};

use crate::{chapter::Chapter, types::Thread};

const URL: &str = "https://mangasee123.com/";

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
    pub async fn chapters_urls(
        &self,
        threads: Thread,
        unicode: bool,
        chapters: Vec<Chapter>,
    ) -> Vec<String> {
        // Set progress bar
        let m = MultiProgress::new();

        let sty = if unicode {
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
            )
            .unwrap()
        } else {
            ProgressStyle::with_template(
                "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
            )
            .expect("Failed to create progress style")
            .progress_chars("#>-")
        };

        let mut urls = Vec::new();

        // Split urls into <threads> parts.
        let mut chapters_split = Vec::new();
        for _ in 0..threads.get() {
            chapters_split.push(Vec::new());
        }
        for (i, url) in chapters.iter().enumerate() {
            chapters_split[i % threads.get() as usize].push(url.clone());
        }

        let mut chapter_for_progress = chapters_split.clone();

        let mut progress_bars = Vec::new();
        progress_bars.push(
            m.add(ProgressBar::new(
                chapter_for_progress
                    .pop()
                    .expect("failed to get previous url")
                    .len() as u64,
            )),
        );

        for _ in 0..threads.get() - 1 {
            progress_bars.push(
                m.insert_after(
                    &progress_bars
                        .clone()
                        .pop()
                        .expect("failed to get previous bar to insert after"),
                    ProgressBar::new(
                        chapter_for_progress
                            .pop()
                            .expect("failed to get url len for bar")
                            .len() as u64,
                    ),
                ),
            );
        }

        for (i, bar) in progress_bars.iter().enumerate() {
            bar.set_style(sty.clone());
            bar.set_message(format!("Chapter's Urls Part {}", i + 1));
        }

        // Spawn a tread for each chunk
        let mut handles = Vec::new();
        for (chunk, bar) in chapters_split.into_iter().zip(progress_bars) {
            let myself = self.clone();
            let chunk = chunk.clone();
            let handle = tokio::spawn(async move {
                {
                    let mut chunk_urls = Vec::new();

                    for chapter in chunk {
                        bar.inc(1);
                        let url = chapter.cur_path_name(myself.i.as_str()).await;
                        for page in 1..chapter.Page.parse::<usize>().unwrap() {
                            chunk_urls.push(format!(
                                "https://{}/manga/{}{}/{:0>4}-{:0>3}.png",
                                url,
                                &myself.i,
                                chapter.directory(),
                                chapter.to_url_id(),
                                page
                            ));
                        }
                    }
                    chunk_urls
                }
            });
            handles.push(handle);
        }

        // Join all the handles
        for handle in handles {
            let mut one_chapters_urls = handle.await.unwrap();
            urls.append(&mut one_chapters_urls);
        }
        urls.par_sort();
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
        )
        .unwrap())
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
