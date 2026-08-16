use colored::Colorize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

const ACCENT: (u8, u8, u8) = (153, 255, 255); 
const DIM: (u8, u8, u8) = (138, 138, 138);
const SUCCESS: (u8, u8, u8) = (95, 215, 107);
const ERROR: (u8, u8, u8) = (255, 95, 95);
const LABEL: (u8, u8, u8) = (224, 224, 224);
const KEY_BG: (u8, u8, u8) = (42, 34, 0);
const RULE_COLOR: (u8, u8, u8) = (68, 68, 68);

fn accent(s: &str) -> String {
    s.truecolor(ACCENT.0, ACCENT.1, ACCENT.2).bold().to_string()
}
fn dim(s: &str) -> String {
    s.truecolor(DIM.0, DIM.1, DIM.2).to_string()
}
fn success(s: &str) -> String {
    s.truecolor(SUCCESS.0, SUCCESS.1, SUCCESS.2).bold().to_string()
}
fn error(s: &str) -> String {
    s.truecolor(ERROR.0, ERROR.1, ERROR.2).bold().to_string()
}
fn label(s: &str) -> String {
    s.truecolor(LABEL.0, LABEL.1, LABEL.2).bold().to_string()
}
fn key(s: &str) -> String {
    s.truecolor(ACCENT.0, ACCENT.1, ACCENT.2)
        .on_truecolor(KEY_BG.0, KEY_BG.1, KEY_BG.2)
        .bold()
        .to_string()
}
fn rule() {
    println!(
        "{}",
        "_".repeat(78).truecolor(RULE_COLOR.0, RULE_COLOR.1, RULE_COLOR.2)
    );
}

const BANNER: &str = r#"
    :::::::-.   :::    :::::::..   .::::::. 
    ;;,   `';, ;;;    ;;;;``;;;; ;;;`    ` 
    `[[     [[ [[[     [[[,/[[[' '[==/[[[[,
    $$,    $$ $$'     $$$$$$c     '''    $
    888_,o8P'o88oo,.__888b "88bo,88b    dP
    MMMMP"`  """"YUMMMMMMM   "W"  "YMmMY" 
    a pretty yt-dlp downloader
"#;

type Favorites = BTreeMap<String, String>;

fn config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config")
        .join("dlrs")
        .join("favorites.json")
}

fn load_favorites() -> Favorites {
    let path = config_path();
    if let Ok(text) = std::fs::read_to_string(&path) {
        if let Ok(Value::Object(map)) = serde_json::from_str::<Value>(&text) {
            return map
                .into_iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k, s.to_string())))
                .collect();
        }
    }
    Favorites::new()
}

fn save_favorites(favs: &Favorites) {
    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(text) = serde_json::to_string_pretty(favs) {
        let _ = std::fs::write(&path, text);
    }
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    let _ = io::stdout().flush();
}

fn header() {
    clear_screen();
    println!("{}", accent(BANNER));
    rule();
    println!();
}

fn prompt(msg: &str) -> String {
    print!("{}", msg);
    let _ = io::stdout().flush();
    let mut buf = String::new();
    if io::stdin().read_line(&mut buf).is_err() {
        return String::new();
    }
    buf.trim().to_string()
}

fn prompt_arrow() -> String {
    prompt(&format!("  {} ", accent(">")))
}

fn show_favorites(favs: &Favorites) {
    if favs.is_empty() {
        println!("  {}\n", dim("no favorites yet"));
        return;
    }
    for (k, path) in favs.iter() {
        println!(" {} {}", key(&format!(" {} ", k)), dim(path));
    }
    println!();
}

fn pick_folder(favs: &mut Favorites) -> Option<String> {
    println!("{}", label("🗀  destination folder"));
    println!();
    show_favorites(favs);

    println!(
        "  {}{}{}{}{}",
        dim("type a "),
        accent("favorite key"),
        dim(", paste a "),
        accent("full path"),
        dim(", or type ")
    );
    println!("  {}{}\n", accent("m"), dim(" to manage favorites."));

    loop {
        let raw = prompt_arrow();
        if raw.is_empty() {
            continue;
        }

        if raw.to_lowercase() == "m" {
            manage_favorites(favs);
            header();
            println!("{}\n", label("🗀  destination folder"));
            show_favorites(favs);
            continue;
        }

        if let Some(path) = favs.get(&raw) {
            println!("  {} {}\n", success("✓"), dim(path));
            return Some(path.clone());
        }

        let p = expand_user(&raw);
        if p.is_dir() {
            println!("  {} {}\n", success("✓"), dim(&p.display().to_string()));
            return Some(p.display().to_string());
        }

        println!(
            "  {} {}",
            error("✗"),
            dim("not a valid key or directory. try again.")
        );
    }
}

fn expand_user(raw: &str) -> PathBuf {
    if let Some(stripped) = raw.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped);
        }
    } else if raw == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    }
    PathBuf::from(raw)
}

fn manage_favorites(favs: &mut Favorites) {
    header();
    println!("{}\n", label("☆ manage favorites"));
    show_favorites(favs);

    println!(
        "  {}  {}{}  {}{}   {}{}   {}{}\n",
        dim("Commands:"),
        accent("a"),
        dim(" add  "),
        accent("r"),
        dim(" rename"),
        accent("d"),
        dim(" delete  "),
        accent("q"),
        dim(" back")
    );

    let cmd = prompt_arrow().to_lowercase();

    match cmd.as_str() {
        "a" => {
            let key_in = prompt(&format!(
                "  {}",
                dim("key combination (up to 3 letters!) ")
            ));
            let key_in: String = key_in.chars().take(1).collect();
            let path_in = prompt(&format!("  {}", dim("folder path ")));
            let p = expand_user(path_in.trim());
            if !p.is_dir() {
                println!("  {}", error("✗ directory does not exist"));
            } else if key_in.is_empty() {
                println!("  {}", error("✗ key cannot be empty"));
            } else {
                favs.insert(key_in.clone(), p.display().to_string());
                save_favorites(favs);
                println!(
                    "  {}{}{}",
                    success("✓ saved "),
                    accent(&key_in),
                    success(&format!(" → {}", p.display()))
                );
            }
        }
        "r" => {
            if favs.is_empty() {
                println!("    {}", error("✗ no favorites to rename"));
            } else {
                let key_in = prompt(&format!("  {}", dim("key to rename ")));
                let key_in: String = key_in.chars().take(3).collect();
                if !favs.contains_key(&key_in) {
                    println!("    {}", error("✗ key not found"));
                } else {
                    println!(
                        "  {}  {}{}  {}{}   {}{}",
                        dim("What would you like to change? "),
                        accent("k"),
                        dim(" key  "),
                        accent("p"),
                        dim(" path  "),
                        accent("b"),
                        dim(" both")
                    );
                    let what = prompt_arrow().to_lowercase();

                    let mut new_key = key_in.clone();
                    let mut new_path = favs.get(&key_in).cloned().unwrap_or_default();

                    let mut aborted = false;

                    if what == "k" || what == "b" {
                        let nk = prompt(&format!("  {}", dim("new key (up to 3 letters!) ")));
                        let nk: String = nk.chars().take(3).collect();
                        if nk.is_empty() {
                            println!("  {}", error("✗ key cannot be empty"));
                            sleep(Duration::from_millis(900));
                            aborted = true;
                        } else if nk != key_in && favs.contains_key(&nk) {
                            println!("  {}", error(&format!("✗ key '{}' already exists", nk)));
                            sleep(Duration::from_millis(900));
                            aborted = true;
                        } else {
                            new_key = nk;
                        }
                    }

                    if !aborted && (what == "p" || what == "b") {
                        let raw_path = prompt(&format!("  {}", dim("new folder path ")));
                        let p = expand_user(raw_path.trim());
                        if !p.is_dir() {
                            println!("  {}", error("✗ directory does not exist"));
                            sleep(Duration::from_millis(900));
                            aborted = true;
                        } else {
                            new_path = p.display().to_string();
                        }
                    }

                    if !aborted {
                        if what != "k" && what != "p" && what != "b" {
                            println!("  {}", error("✗ unknown option"));
                        } else {
                            favs.remove(&key_in);
                            favs.insert(new_key.clone(), new_path.clone());
                            save_favorites(favs);
                            println!(
                                "  {}{}{}",
                                success("✓ updated "),
                                accent(&new_key),
                                success(&format!(" → {}", new_path))
                            );
                        }
                    }
                }
            }
        }
        "d" => {
            let key_in = prompt(&format!("  {}", dim("key to remove ")));
            let key_in: String = key_in.chars().take(1).collect();
            if favs.remove(&key_in).is_some() {
                save_favorites(favs);
                println!("  {}", success("✓ removed"));
            } else {
                println!("  {}", error("✗ key not found"));
            }
        }
        _ => {}
    }

    sleep(Duration::from_millis(900));
}

fn run_download(url: &str, mode: &str, folder: &str) {
    let mut cmd_args: Vec<String> = vec!["--no-playlist".to_string()];

    if mode == "a" {
        cmd_args.extend(
            [
                "-f",
                "bestaudio/best",
                "-x",
                "--audio-format",
                "mp3",
                "--audio-quality",
                "0",
                "--embed-thumbnail",
                "--add-metadata",
            ]
            .iter()
            .map(|s| s.to_string()),
        );
    } else {
        cmd_args.extend(
            [
                "-f",
                "bestvideo+bestaudio/best",
                "--merge-output-format",
                "mp4",
                "--embed-thumbnail",
                "--add-metadata",
            ]
            .iter()
            .map(|s| s.to_string()),
        );
    }

    let output_template = Path::new(folder)
        .join("%(title)s.%(ext)s")
        .display()
        .to_string();
    cmd_args.push("-o".to_string());
    cmd_args.push(output_template);
    cmd_args.push(url.to_string());

    rule();
    println!();

    let full_cmd = format!("yt-dlp {}", cmd_args.join(" "));
    print_panel(&full_cmd, "running", RULE_COLOR);
    println!();

    let status = Command::new("yt-dlp").args(&cmd_args).status();

    println!();
    match status {
        Ok(s) if s.success() => {
            print_panel("  download complete  ", "", SUCCESS);
        }
        _ => {
            print_panel(
                "  yt-dlp exited with errors. see output above.  ",
                "",
                ERROR,
            );
        }
    }
}

/// A very small stand-in for rich's `Panel.fit`.
fn print_panel(body: &str, title: &str, border_color: (u8, u8, u8)) {
    let border = |s: &str| s.truecolor(border_color.0, border_color.1, border_color.2);
    let width = body.chars().count().max(title.chars().count() + 4) + 2;
    let top = if title.is_empty() {
        format!("╭{}╮", "─".repeat(width))
    } else {
        let t = format!(" {} ", title);
        let remaining = width.saturating_sub(t.chars().count());
        format!(
            "╭{}{}{}╮",
            "─".repeat(remaining / 2),
            t,
            "─".repeat(remaining - remaining / 2)
        )
    };
    println!("{}", border(&top));
    println!("{} {} {}", border("│"), dim(body), border("│"));
    println!("{}", border(&format!("╰{}╯", "─".repeat(width))));
}

fn yt_dlp_available() -> bool {
    Command::new("which")
        .arg("yt-dlp")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn main() {
    if !yt_dlp_available() {
        println!(
            "\n{} {}\n",
            error("yt-dlp is not installed."),
            format!(
                "{} {}",
                dim("run:"),
                accent("pip install yt-dlp")
            )
        );
        std::process::exit(1);
    }

    let mut favs = load_favorites();

    loop {
        header();

        println!(
            "{}  {}{}{}{}{}\n",
            label("paste your URL"),
            dim("("),
            accent("q"),
            dim(" to quit, "),
            accent("m"),
            dim(" to manage favorites)")
        );
        let url = prompt_arrow();

        if matches!(url.to_lowercase().as_str(), "q" | "quit" | "exit") {
            println!("\n  {}\n", dim("bye"));
            break;
        } else if matches!(url.to_lowercase().as_str(), "m" | "manage" | "favorites") {
            manage_favorites(&mut favs);
        }
        if url.is_empty() {
            continue;
        }

        println!();
        println!("{}\n", label("audio or video?"));
        println!(
            "   {}   {}",
            format!("{}  audio (mp3)", accent("a")),
            format!("{}  video (mp4)", accent("v"))
        );
        println!();

        let mode = loop {
            let m = prompt_arrow().to_lowercase();
            if m == "a" || m == "v" {
                break m;
            }
            println!(
                "  {}{}{}{}",
                error("type "),
                accent("a"),
                error(" or "),
                accent("v")
            );
        };

        println!();
        let folder = match pick_folder(&mut favs) {
            Some(f) => f,
            None => continue,
        };

        run_download(&url, &mode, &folder);

        println!();
        let again = prompt(&format!(
            "  {} {}{}{}",
            dim("download another?"),
            accent("y"),
            dim("/"),
            accent("n")
        ))
        .to_lowercase();
        if again != "y" {
            println!("\n  {}\n", dim("bye"));
            break;
        }
    }
}