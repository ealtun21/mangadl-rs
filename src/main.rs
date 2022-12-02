use crossterm::style::Stylize;
use std::time::Duration;

use inquire::{
    ui::{Color, RenderConfig, StyleSheet, Styled},
    CustomType, InquireError, MultiSelect, Select,
};
use mangadl_rs::{
    args::{display_help, get_encoding, Encoding},
    chapter::Chapter,
    fetch,
    manga::Manga,
    types::{DownloadType, SaveType, Thread},
};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    let encoding = get_encoding(&args);
    if display_help(&args) {
        return Ok(());
    }

    if encoding == Encoding::Unicode {
        inquire::set_global_render_config(get_render_config());
    }

    let future_manga = tokio::spawn(async move {
        loop {
            match Manga::all_manga_list().await {
                Ok(manga) => break manga,
                Err(e) => {
                    eprintln!("Error: {}, Retrying!", e);
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        }
    });

    let save_type = loop {
        match Select::new(
            "How would you like to save?",
            vec![
                SaveType::PdfSplit,
                SaveType::PdfSingle,
                SaveType::Images,
                SaveType::ImagesChapter,
                SaveType::Urls,
            ],
        )
        .prompt()
        {
            Ok(k) => break k,
            Err(InquireError::OperationInterrupted) => return Ok(()),
            Err(_) => eprintln!("{}", "Please select an option.".red().slow_blink()),
        }
    };

    let download_type = match save_type {
        SaveType::Urls => DownloadType::Single,
        SaveType::ImagesChapter | SaveType::Images | SaveType::PdfSingle | SaveType::PdfSplit => {
            loop {
                match Select::new(
                    "How would you like to download?",
                    vec![DownloadType::Single, DownloadType::Multi],
                )
                .prompt()
                {
                    Ok(ans) => break ans,
                    Err(InquireError::OperationInterrupted) => return Ok(()),
                    Err(_) => eprintln!("{}", "Please select an option.".red().slow_blink()),
                }
            }
        }
    };

    let mut treads: Thread = Thread::new(1).unwrap();

    if let SaveType::Urls = save_type {
        treads = loop {
            match CustomType::new("Number of Threads: ")
                .with_error_message("Please type a valid number")
                .with_help_message("Type the amount of threads you want to use")
                .prompt()
            {
                Ok(ans) => break ans,
                Err(InquireError::OperationInterrupted) => return Ok(()),
                Err(_) => eprintln!("{}", "Please enter amount of threads".red().slow_blink()),
            }
        };
    }

    if let DownloadType::Multi = download_type {
        treads = loop {
            match CustomType::new("Number of Threads: ")
                .with_error_message("Please type a valid number")
                .with_help_message("Type the amount of threads you want to use")
                .prompt()
            {
                Ok(ans) => break ans,
                Err(InquireError::OperationInterrupted) => return Ok(()),
                Err(_) => eprintln!("{}", "Please enter amount of threads".red().slow_blink()),
            }
        };
    }

    let manga = future_manga.await.unwrap();

    let genres =
        loop {
            match MultiSelect::new("Select Genre(s)", Manga::find_all_genre(&manga))
            .with_help_message(
                "esc to skip, ↑↓ to move, space to select one, → to all, ← to none, type to filter",
            )
            .prompt_skippable()
        {
            Ok(ans) => if ans.as_ref().is_some() && ans.as_ref().unwrap().is_empty() {
                eprintln!(
                    "{}",
                    "Please skip with the ESC button or Select Genre(s)"
                        .red()
                        .slow_blink()
                );
                continue;
            } else {
                break ans
            },
            Err(InquireError::OperationInterrupted) => return Ok(()),
            Err(_) => continue,
        }
        };

    let ans = if let Some(..) = genres {
        let selection_manga = Manga::filter_manga(genres.unwrap(), manga);
        if selection_manga.is_none() || selection_manga.clone().unwrap().is_empty() {
            println!(
                "{}",
                "No manga with such genres found, closing program"
                    .blue()
                    .rapid_blink()
            );
            sleep(Duration::from_secs(3)).await;
            return Ok(());
        }
        loop {
            match Select::new("Select Manga", selection_manga.as_ref().unwrap().clone()).prompt() {
                Ok(ans) => break ans,
                Err(InquireError::OperationInterrupted) => return Ok(()),
                Err(_) => eprintln!("{}", "Please select manga".red().slow_blink()),
            }
        }
    } else {
        loop {
            match Select::new("Select Manga", manga.clone()).prompt() {
                Ok(ans) => break ans,
                Err(InquireError::OperationInterrupted) => return Ok(()),
                Err(_) => eprintln!("{}", "Please select manga".red().slow_blink()),
            }
        }
    };

    let chapters = loop {
        if let Ok(chapters) = MultiSelect::new(
            "Select Chapters",
            Chapter::list(&ans.i, &ans.l)
                .await
                .expect("Network Error, try again later."),
        )
        .prompt()
        {
            if chapters.is_empty() {
                eprintln!(
                    "{}",
                    "Please select at least one chapter".red().slow_blink()
                );
                continue;
            }
            break chapters;
        }
        eprintln!("{}", "Please select a chapter".red().slow_blink());
    };

    fetch::download_manga(
        ans,
        chapters,
        save_type,
        download_type,
        treads,
        encoding == Encoding::Unicode,
    )
    .await;

    Ok(())
}

fn get_render_config() -> RenderConfig {
    let mut render_config = RenderConfig::default();
    render_config.prompt_prefix = Styled::new("?").with_fg(Color::DarkGreen);
    render_config.highlighted_option_prefix = Styled::new("➠").with_fg(Color::LightBlue);
    render_config.selected_checkbox = Styled::new("☑").with_fg(Color::LightGreen);
    render_config.scroll_up_prefix = Styled::new("⇞");
    render_config.scroll_down_prefix = Styled::new("⇟");
    render_config.unselected_checkbox = Styled::new("☐");

    render_config.error_message = render_config
        .error_message
        .with_prefix(Styled::new("❌").with_fg(Color::LightRed));
    render_config.help_message = StyleSheet::new().with_fg(Color::DarkMagenta);

    render_config
}
