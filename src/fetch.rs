use std::{fs::File, path::Path};

use image::DynamicImage;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::io::Write;
use tokio::fs;

use crate::{
    chapter::Chapter,
    manga::Manga,
    types::{DownloadType, SaveType},
};

pub async fn get_img(url: &str) -> Result<DynamicImage, reqwest::Error> {
    Ok(
        image::load_from_memory(&reqwest::get(url).await?.bytes().await?)
            .expect("Failed to load image"),
    )
}

pub async fn download_manga(
    manga: Manga,
    chapters: Vec<Chapter>,
    save_type: SaveType,
    download_type: DownloadType,
    threads: u8,
) {
    let urls = manga.chapters_urls_multi(chapters).await;
    match (save_type, download_type) {
        (SaveType::Urls, _) => urls_download(urls, &manga).await,
        (SaveType::Images, DownloadType::Single) => {
            images_download_single_unicode(urls, &manga).await
        } //images_download_single(urls, &manga).await
        (SaveType::Images, DownloadType::Multi) => {
            images_download_multi_unicode(urls, &manga, threads.into()).await;
            //images_download_multi(urls, &manga, threads.into()).await // For windows
        }
    }
}

// Download urls seperated by a a line into a 1 text file named the manga.
pub async fn urls_download(urls: Vec<String>, manga: &Manga) {
    let mut file = File::create(format!("{}.txt", manga.i)).expect("Failed to create file");
    for url in urls {
        file.write_all(format!("{}\n", url).as_bytes())
            .expect("Failed to write to file");
    }
}

pub async fn images_download_single(urls: Vec<String>, manga: &Manga) {
    images_download_multi(urls, manga, 1).await;
}

pub async fn images_download_single_unicode(urls: Vec<String>, manga: &Manga) {
    images_download_multi_unicode(urls, manga, 1).await;
}

pub async fn images_download_multi(urls: Vec<String>, manga: &Manga, threads: usize) {
    // Set progress bar
    let m = MultiProgress::new();
    let sty = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
    )
    .expect("Failed to create progress style")
    .progress_chars("#>-");

    // Split urls into 16 parts.
    let mut urls_split = Vec::new();
    for _ in 0..threads {
        urls_split.push(Vec::new());
    }
    for (i, url) in urls.iter().enumerate() {
        urls_split[i % threads].push(url.clone());
    }

    let mut urls_for_progress = urls_split.clone();

    let mut progress_bars = Vec::new();
    progress_bars.push(
        m.add(ProgressBar::new(
            urls_for_progress
                .pop()
                .expect("failed to get previous url")
                .len() as u64,
        )),
    );
    for _ in 0..threads - 1 {
        progress_bars.push(
            m.insert_after(
                &progress_bars
                    .clone()
                    .pop()
                    .expect("failed to get previous bar to insert after"),
                ProgressBar::new(
                    urls_for_progress
                        .pop()
                        .expect("failed to get url len for bar")
                        .len() as u64,
                ),
            ),
        );
    }
    for (i, bar) in progress_bars.iter().enumerate() {
        bar.set_style(sty.clone());
        bar.set_message(format!("Part {}", i + 1));
    }

    // Spawn a threads for each part.
    let mut handles = Vec::new();
    for (urls, bar) in urls_split.into_iter().zip(progress_bars) {
        let manga = manga.clone();
        let handle = tokio::spawn(async move {
            for url in urls {
                bar.inc(1);
                let img = loop {
                    match get_img(url.as_str()).await {
                        Ok(img) => break img,
                        Err(e) => {
                            eprintln!("{e}\nFailed to download image, Retrying...");
                            std::thread::sleep(std::time::Duration::from_millis(100));
                            continue;
                        }
                    }
                };
                let path = format!(
                    "{name}/{page}",
                    name = &manga.i,
                    page = &url.split('/').last().unwrap(),
                );
                let file_path = Path::new(&path);
                fs::create_dir_all(file_path.parent().unwrap())
                    .await
                    .unwrap();
                img.save(file_path).expect("Failed to save image");
            }
        });
        handles.push(handle);
    }
    // Wait for all threads to finish.
    for handle in handles {
        handle.await.unwrap();
    }
}

pub async fn images_download_multi_unicode(urls: Vec<String>, manga: &Manga, threads: usize) {
    // Set progress bar
    let m = MultiProgress::new();
    let sty = ProgressStyle::with_template(
        "{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
    )
    .unwrap();

    // Split urls into 16 parts.
    let mut urls_split = Vec::new();
    for _ in 0..threads {
        urls_split.push(Vec::new());
    }
    for (i, url) in urls.iter().enumerate() {
        urls_split[i % threads].push(url.clone());
    }

    let mut urls_for_progress = urls_split.clone();

    let mut progress_bars = Vec::new();
    progress_bars.push(
        m.add(ProgressBar::new(
            urls_for_progress
                .pop()
                .expect("failed to get previous url")
                .len() as u64,
        )),
    );
    for _ in 0..threads - 1 {
        progress_bars.push(
            m.insert_after(
                &progress_bars
                    .clone()
                    .pop()
                    .expect("failed to get previous bar to insert after"),
                ProgressBar::new(
                    urls_for_progress
                        .pop()
                        .expect("failed to get url len for bar")
                        .len() as u64,
                ),
            ),
        );
    }
    for (i, bar) in progress_bars.iter().enumerate() {
        bar.set_style(sty.clone());
        bar.set_message(format!("Part {}", i + 1));
    }

    // Spawn a threads for each part.
    let mut handles = Vec::new();
    for (urls, bar) in urls_split.into_iter().zip(progress_bars) {
        let manga = manga.clone();
        let handle = tokio::spawn(async move {
            for url in urls {
                bar.inc(1);
                let img = loop {
                    match get_img(url.as_str()).await {
                        Ok(img) => break img,
                        Err(e) => {
                            eprintln!("{e}\nFailed to download image, Retrying...");
                            std::thread::sleep(std::time::Duration::from_millis(500));
                            continue;
                        }
                    }
                };
                let path = format!(
                    "{name}/{page}",
                    name = &manga.i,
                    page = &url.split('/').last().unwrap(),
                );
                let file_path = Path::new(&path);
                fs::create_dir_all(file_path.parent().unwrap())
                    .await
                    .unwrap();
                img.save(file_path).expect("Failed to save image");
            }
        });
        handles.push(handle);
    }
    // Wait for all threads to finish.
    for handle in handles {
        handle.await.unwrap();
    }
}
