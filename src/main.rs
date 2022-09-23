use std::time::Duration;
use crossterm::style::Stylize;

use inquire::{
    ui::{Color, RenderConfig, StyleSheet, Styled},
    CustomType, MultiSelect, Select,
};
use mangadl_rs::{
    chapter::Chapter,
    fetch,
    manga::Manga,
    types::{DownloadType, SaveType},
};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use theme only for linux builds. Comment out for windows builds.
    inquire::set_global_render_config(get_render_config());


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
        if let Ok(save_type) = Select::new(
            "How would you like to save?",
            vec![SaveType::Urls, SaveType::Images],
        )
        .prompt()
        {
            break save_type;
        }
        eprintln!("{}", "Please select an option.".red().slow_blink());
    };

    let download_type = match save_type {
        SaveType::Urls => DownloadType::Single,
        SaveType::Images => loop {
            if let Ok(download_type) = Select::new(
                "How would you like to download?",
                vec![DownloadType::Single, DownloadType::Multi],
            )
            .prompt()
            {
                break download_type;
            }
            eprintln!("{}", "Please select an option.".red().slow_blink());
        },
    };

    let mut treads: u8 = 1;

    if let DownloadType::Multi = download_type {
        treads = loop {
            if let Ok(treads) = CustomType::new("Number of Threads: ")
                .with_error_message("Please type a valid number")
                .with_help_message("Type the amount of threads you want to use")
                .prompt()
            {
                break treads;
            }
            eprintln!("{}", "Please enter amount of threads".red().slow_blink());
        };
    }

    let manga = future_manga.await.unwrap();

    let genres = loop {
        if let Ok(genres) = MultiSelect::new(
            &format!("Select Genre(s) {}","Skip with ESC".yellow().italic()),
            Manga::find_all_genre(&manga).await,
        )
        .prompt_skippable()
        {
            if genres.is_some() && genres.clone().unwrap().is_empty() {
                eprintln!(
                    "{}",
                    "Please skip with the ESC button or Select Genre(s)".red().slow_blink()
                );
                continue;
            }
            break genres;
        }
    };

    let ans = if let Some(..) = genres {
        let selection_manga = Manga::filter_manga(genres.unwrap(), manga);
        if selection_manga.is_none() || selection_manga.clone().unwrap().is_empty() {
            println!("{}", "No manga with such genres found, closing program".blue().rapid_blink());
            sleep(Duration::from_secs(3)).await;
            // Restart?
            return Ok(());
        }
        loop {
            if let Ok(ans) =
                Select::new("Select Manga", selection_manga.as_ref().unwrap().clone()).prompt()
            {
                break ans;
            }
            eprintln!("{}", "Please select manga".red().slow_blink());
        }
    } else {
        loop {
            if let Ok(ans) = Select::new("Select Manga", manga.clone()).prompt() {
                break ans;
            }
            eprintln!("{}", "Please select manga".red().slow_blink());
        }
    };

    let chapters = loop {
        if let Ok(chapters) = MultiSelect::new(
            "Select Chapters",
            Chapter::list(&ans.i)
                .await
                .expect("Network Error, try again later."),
        )
        .prompt()
        {
            if chapters.is_empty() {
                eprintln!("{}", "Please select at least one chapter".red().slow_blink());
                continue;
            }
            break chapters;
        }
        eprintln!("{}", "Please select a chapter".red().slow_blink());
    };

    fetch::download_manga(ans, chapters, save_type, download_type, treads, true).await;
    //fetch::download_manga(ans, chapters, save_type, download_type, treads, false).await; // For windows builds
    
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
