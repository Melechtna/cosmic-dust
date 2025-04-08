use cosmic::app::{run, Settings as CosmicSettings};
use ui::CosmicDust;
use std::env;
use clap::Parser;

mod files;
mod disk;
mod partition;
mod progress_bar;
mod sizes;
mod ui;
mod crawler;

#[derive(Parser, Debug)]
#[command(version, about = "A disk usage analyzer for COSMIC DE", long_about = None)]
struct Args {
    /// Enable verbose output for debugging
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> cosmic::iced::Result {
    let args = Args::parse();

    // Debug print to check desktop environment
    let desktop = env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
    if args.verbose {
        println!("Desktop: {:?}", desktop);
    }

    // Convert to lowercase for easier matching
    let desktop_lower = desktop.to_lowercase();

    // Create cosmic settings
    let mut cosmic_settings = CosmicSettings::default()
        .size(cosmic::iced::Size { width: 800.0, height: 600.0 });

    // Set icon theme based on desktop environment to avoid weirdness on other DE's
    if desktop_lower.contains("cosmic") {
        if args.verbose {
            println!("Detected Cosmic DE, using default Cosmic icon theme");
        }
    } else if desktop_lower.contains("kde") || desktop_lower.contains("plasma") || desktop_lower.contains("lxqt") {
        cosmic_settings = cosmic_settings.default_icon_theme("breeze");
        if args.verbose {
            println!("Detected Qt-based DE (KDE/LXQt), using Breeze icon theme");
        }
    } else if desktop_lower.contains("deepin") {
        cosmic_settings = cosmic_settings.default_icon_theme("deepin");
        if args.verbose {
            println!("Detected Deepin DE, using Deepin icon theme");
        }
    } else if desktop_lower.contains("gnome") {
        cosmic_settings = cosmic_settings.default_icon_theme("Adwaita");
        if args.verbose {
            println!("Detected GNOME, using Adwaita icon theme");
        }
    } else if desktop_lower.contains("mate") {
        cosmic_settings = cosmic_settings.default_icon_theme("mate");
        if args.verbose {
            println!("Detected MATE, using MATE icon theme");
        }
    } else if desktop_lower.contains("cinnamon") {
        cosmic_settings = cosmic_settings.default_icon_theme("mint-x");
        if args.verbose {
            println!("Detected Cinnamon, using Mint-X icon theme");
        }
    } else if desktop_lower.contains("budgie") {
        cosmic_settings = cosmic_settings.default_icon_theme("Adwaita");
        if args.verbose {
            println!("Detected Budgie, using Adwaita icon theme");
        }
    } else if desktop_lower.contains("lxde") {
        cosmic_settings = cosmic_settings.default_icon_theme("papirus");
        if args.verbose {
            println!("Detected LXDE, using Papirus icon theme");
        }
    } else {
        cosmic_settings = cosmic_settings.default_icon_theme("breeze");
        if args.verbose {
            println!("Unrecognized DE, defaulting to Breeze icon theme");
        }
    }

    run::<CosmicDust>(cosmic_settings, args.verbose)
}