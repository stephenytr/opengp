use clap::{Parser, Subcommand};
use opengp_theme_converter::{
    check_contrast, map_alacritty_to_opengp, parse_by_extension, render_opengp_toml,
};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "opengp_theme_converter")]
#[command(about = "Convert Alacritty themes to OpenGP format")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Convert {
        #[arg(short, long)]
        input: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    Validate {
        #[arg(short, long)]
        file: PathBuf,
    },
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Convert { input, output } => {
            let content = std::fs::read_to_string(&input)?;
            let alacritty_theme = parse_by_extension(&input, &content)?;
            let opengp_theme = map_alacritty_to_opengp(&alacritty_theme);
            let toml = render_opengp_toml(&opengp_theme)?;

            match output {
                Some(path) => std::fs::write(&path, &toml)?,
                None => println!("{toml}"),
            }
        }
        Commands::Validate { file } => {
            let content = std::fs::read_to_string(&file)?;
            let alacritty_theme = parse_by_extension(&file, &content)?;
            let opengp_theme = map_alacritty_to_opengp(&alacritty_theme);
            println!("Valid theme: {:?}", opengp_theme.schema_version);
        }
    }
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
