//! asscify — beautiful terminal UI for turning images into ASCII art.

mod app;
mod ascii;
mod scanner;
mod ui;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use crate::app::App;
use crate::ascii::{convert_path, showcase_art, AsciiOptions, AsciiStyle};

#[derive(Parser, Debug)]
#[command(
    name = "asscify",
    about = "Beautiful terminal gallery that turns images into ASCII art",
    long_about = "Open a TUI gallery of images in the current directory, preview \
                  live ASCII conversions, flip styles, and drop into a fullscreen showcase.\n\n\
                  Run with no args to launch the UI."
)]
struct Cli {
    /// Directory to scan (defaults to cwd)
    #[arg(short, long, global = true)]
    dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Launch the interactive TUI (default)
    Ui,
    /// Print ASCII art for one image to stdout
    Convert {
        /// Image path
        path: PathBuf,
        /// Output width in characters
        #[arg(short, long, default_value_t = 80)]
        width: u32,
        /// Style: classic, blocks, detailed, minimal, binary
        #[arg(short, long, default_value = "classic")]
        style: String,
        /// Invert luminance
        #[arg(long)]
        invert: bool,
        /// Disable ANSI color escapes
        #[arg(long)]
        no_color: bool,
        /// Write to file instead of stdout
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Print a procedural ASCII showcase orb
    Showcase {
        #[arg(short, long, default_value_t = 72)]
        width: u32,
        #[arg(short, long, default_value = "blocks")]
        style: String,
    },
}

fn parse_style(s: &str) -> anyhow::Result<AsciiStyle> {
    match s.to_ascii_lowercase().as_str() {
        "classic" => Ok(AsciiStyle::Classic),
        "blocks" | "block" => Ok(AsciiStyle::Blocks),
        "detailed" | "detail" => Ok(AsciiStyle::Detailed),
        "minimal" | "mini" => Ok(AsciiStyle::Minimal),
        "binary" | "bin" => Ok(AsciiStyle::Binary),
        other => anyhow::bail!(
            "unknown style '{other}' — try classic, blocks, detailed, minimal, binary"
        ),
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let dir = cli
        .dir
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    match cli.command.unwrap_or(Commands::Ui) {
        Commands::Ui => run_ui(dir),
        Commands::Convert {
            path,
            width,
            style,
            invert,
            no_color,
            output,
        } => {
            let opts = AsciiOptions {
                width,
                style: parse_style(&style)?,
                invert,
                color: !no_color,
                ..AsciiOptions::default()
            };
            let art = convert_path(&path, &opts)?;
            let plain = art.as_plain_string();
            if let Some(out) = output {
                std::fs::write(out, plain)?;
            } else if opts.color {
                print_colored(&art);
            } else {
                print!("{plain}");
            }
            Ok(())
        }
        Commands::Showcase { width, style } => {
            let style = parse_style(&style)?;
            let art = showcase_art(width, style, true);
            print_colored(&art);
            Ok(())
        }
    }
}

fn print_colored(art: &crate::ascii::AsciiArt) {
    for row in &art.rows {
        for cell in row {
            match cell.color {
                ratatui::style::Color::Rgb(r, g, b) => {
                    print!("\x1b[38;2;{r};{g};{b}m{}", cell.ch);
                }
                _ => print!("{}", cell.ch),
            }
        }
        println!("\x1b[0m");
    }
}

fn run_ui(dir: PathBuf) -> anyhow::Result<()> {
    let mut terminal = ratatui::init();
    let result = (|| {
        let app = App::new(dir)?;
        app.run(&mut terminal)
    })();
    ratatui::restore();
    result
}
