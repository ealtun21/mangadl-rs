use std::{collections::BTreeMap, fs::File, io::BufWriter, path::Path};

use image_to_pdf::ImageToPdf;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use printpdf::image_crate::DynamicImage;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
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
    unicode: bool,
) {
    println!("Fetching urls...");
    let urls = manga.chapters_urls(threads.into(), unicode, chapters).await;
    match (save_type, download_type) {
        (SaveType::Urls, _) => urls_download(urls, &manga),
        (SaveType::Images, DownloadType::Single) => {
            images_download(false, unicode, urls, &manga, 1).await;
        }
        (SaveType::Images, DownloadType::Multi) => {
            images_download(false, unicode, urls, &manga, threads.into()).await;
        }
        (SaveType::PdfSingle, DownloadType::Single) => {
            save_to_pdf(download_to_ram(unicode, urls, 1).await, &manga, unicode).await;
        }
        (SaveType::PdfSingle, DownloadType::Multi) => {
            save_to_pdf(
                download_to_ram(unicode, urls, threads.into()).await,
                &manga,
                unicode,
            )
            .await;
        }
        (SaveType::PdfSplit, DownloadType::Single) => {
            save_to_pdf_split_chapters(download_to_ram(unicode, urls, 1).await, &manga, unicode);
        }
        (SaveType::PdfSplit, DownloadType::Multi) => {
            save_to_pdf_split_chapters(
                download_to_ram(unicode, urls, threads.into()).await,
                &manga,
                unicode,
            );
        }
        (SaveType::ImagesChapter, DownloadType::Single) => {
            images_download(true, unicode, urls, &manga, 1).await;
        }
        (SaveType::ImagesChapter, DownloadType::Multi) => {
            images_download(true, unicode, urls, &manga, threads.into()).await;
        }
    }
}

// Download urls seperated by a a line into a 1 text file named the manga.
pub fn urls_download(urls: Vec<String>, manga: &Manga) {
    let mut file = File::create(format!("{}.txt", manga.i)).expect("Failed to create file");
    for url in urls {
        file.write_all(format!("{}\n", url).as_bytes())
            .expect("Failed to write to file");
    }
}

pub async fn images_download(
    folder: bool,
    unicode: bool,
    urls: Vec<String>,
    manga: &Manga,
    threads: usize,
) {
    println!("Downlading images...");
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

    // Split urls into <threads> parts.
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

                let path = if folder {
                    format!(
                        "{name}/{chapter}/{page}",
                        name = &manga.i,
                        chapter = &url.split('/').last().unwrap().split('-').next().unwrap(),
                        page = &url.split('/').last().unwrap().split('-').last().unwrap(),
                    )
                } else {
                    format!(
                        "{name}/{page}",
                        name = &manga.i,
                        page = &url.split('/').last().unwrap(),
                    )
                };

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

pub async fn download_to_ram(
    unicode: bool,
    urls: Vec<String>,
    threads: usize,
) -> BTreeMap<String, DynamicImage> {
    println!("Downloading images to ram...");
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

    // Split urls into <threads> parts.
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

    let mut handles = Vec::new();
    for (urls, bar) in urls_split.into_iter().zip(progress_bars) {
        let handle = tokio::spawn(async move {
            let mut images = BTreeMap::new();
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
                images.insert(url.split('/').last().unwrap().to_string(), img);
            }
            images
        });
        handles.push(handle);
    }
    let mut images = BTreeMap::new();
    // Wait for all threads to finish.
    for handle in handles {
        images.extend(handle.await.unwrap());
    }
    images
}

pub async fn save_to_pdf(images: BTreeMap<String, DynamicImage>, manga: &Manga, unicode: bool) {
    println!("Adding images to a pdf...");
    let out_file = File::create(format!("{}.pdf", manga.i)).unwrap();

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

    let pdf = ImageToPdf::default()
        .add_images_par(images.into_par_iter().map(|(_, img)| img))
        .set_document_title(format!("{}.pdf", manga.i))
        .create_with_progress_first_pdf(sty, m);

    println!("Saving to file...");
    pdf.save(&mut BufWriter::new(out_file)).unwrap();
}

pub fn save_to_pdf_split_chapters(
    images: BTreeMap<String, DynamicImage>,
    manga: &Manga,
    unicode: bool,
) {
    println!("Adding images to pdfs...");
    let mut images_split = BTreeMap::new();
    for (i, img) in images {
        let chapter = i.split('-').next().unwrap();
        images_split
            .entry(chapter.to_string())
            .or_insert_with(Vec::new)
            .push(img);
    }
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

    println!("Saving to file...");
    let progress_bar = ProgressBar::new(images_split.keys().len() as u64).with_style(sty);

    images_split.into_par_iter().for_each(|(chapter, images)| {
        let out_file = File::create(format!("{}-{}.pdf", manga.i, chapter)).unwrap();

        let pdf = ImageToPdf::default()
            .add_images_par(images.into_par_iter())
            .set_document_title(format!("{}-{}.pdf", manga.i, chapter))
            .create_pdf();

        pdf.save(&mut BufWriter::new(out_file)).unwrap();
        progress_bar.inc(1);
    });
}
