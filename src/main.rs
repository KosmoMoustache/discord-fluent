use clap::{Parser, Subcommand};
use std::path::Path;

mod data;
mod generate;
mod utils;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Data {
        #[arg(long, short, value_name = "PATH", default_value = "public/tree.json")]
        path: String,

        #[arg(long = "normal", value_name = "PATH", default_value = "public/fluentui-emoji/assets", value_parser = utils::path_exist)]
        normal_fluent: String,
        #[arg(long = "animated", value_name = "PATH", default_value = "public/fluentui-emoji-animated/assets", value_parser = utils::path_exist)]
        animated_fluent: String,

        #[arg(
            long = "duplication",
            default_value = "false",
            help = "Print which emojis are skipped due to duplication"
        )]
        verbose_duplication: bool,
    },
    Generate {
        #[arg(long, short, value_name = "DATA PATH", default_value = "public/tree.json", value_parser = utils::path_exist)]
        tree: String,

        #[arg(long, short, value_name = "OUT PATH", default_value = "public/css", value_parser = utils::path_exist)]
        out_path: String,

        #[arg(long, short, help = "Generate Discord compatible CSS files")]
        discord: bool,

        #[arg(
            long = "3d",
            requires = "discord",
            value_name = "FILENAME_3D_CSS",
            default_value = "fluent_3d.css"
        )]
        discord_filename_3d: String,
        #[arg(
            long = "color",
            requires = "discord",
            value_name = "FILENAME_COLOR_CSS",
            default_value = "fluent_color.css"
        )]
        discord_filename_color: String,
        #[arg(
            long = "flat",
            requires = "discord",
            value_name = "FILENAME_FLAT_CSS",
            default_value = "fluent_flat.css"
        )]
        discord_filename_flat: String,
        #[arg(
            long = "hc",
            requires = "discord",
            value_name = "FILENAME_HC_CSS",
            default_value = "fluent_high_contrast.css"
        )]
        discord_filename_hc: String,
        #[arg(
            long = "animated",
            requires = "discord",
            value_name = "FILENAME_ANIMATED_CSS",
            default_value = "fluent_animated.css"
        )]
        discord_filename_animated: String,
    },
}

fn main() {
    let args: Cli = Cli::parse();

    match args.command {
        Commands::Data {
            path,
            normal_fluent,
            animated_fluent,
            verbose_duplication,
        } => {
            data::create_glyph_data(
                Path::new(&path),
                Path::new(&normal_fluent),
                Path::new(&animated_fluent),
                verbose_duplication,
            );
        }
        Commands::Generate {
            tree,
            out_path: path,
            discord,
            discord_filename_3d,
            discord_filename_color,
            discord_filename_flat,
            discord_filename_hc,
            discord_filename_animated,
        } => {
            generate::generate(
                Path::new(&tree),
                Path::new(&path),
                discord,
                Some(generate::DiscordFilenames {
                    three_d: &discord_filename_3d,
                    color: &discord_filename_color,
                    flat: &discord_filename_flat,
                    hc: &discord_filename_hc,
                    animated: &discord_filename_animated,
                }),
            );
        }
    }
}
