use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use crate::data::Glyphs;

pub struct DiscordFilenames<'a> {
    pub three_d: &'a str,
    pub color: &'a str,
    pub flat: &'a str,
    pub hc: &'a str,
    pub animated: &'a str,
}

fn create_css_file(path: &Path, filename: &str) -> File {
    File::create(path.join(filename))
        .expect(format!("Failed to create CSS file {}/{}", path.display(), filename).as_ref())
}

pub fn generate(
    tree_path: &Path,
    out_path: &Path,
    discord: bool,
    discord_filenames: Option<DiscordFilenames>,
) {
    let tree_content = fs::read_to_string(tree_path)
        .expect(format!("Failed to read {}", out_path.display()).as_ref());
    let glyphs: Vec<Glyphs> = serde_json::from_str(&tree_content)
        .expect(format!("Failed to create {}", out_path.display()).as_ref());

    gen_css_default(&glyphs, out_path);

    if discord && discord_filenames.is_some() {
        gen_css_discord(&glyphs, out_path, discord_filenames.unwrap());
    }
}

fn gen_css_default(glyphs: &Vec<Glyphs>, path: &Path) {
    let mut file = create_css_file(path, "fluent_default.css");

    for _glyph in glyphs {
        let glyph = _glyph.glyph.as_str();
        file.write_all(format!(".{}:before {{ content: \"{}\"; }}\n", glyph, glyph).as_bytes())
            .unwrap();
    }

    println!("CSS generated at {:?}", path.join("fluent_default.css"));
}

fn gen_css_discord(glyphs: &Vec<Glyphs>, path: &Path, discord_filenames: DiscordFilenames) {
    fn _write_css(file: &mut File, glyph: &str, url: Option<&String>) {
        if let Some(u) = url {
            file.write_all(
                format!("img[alt|=\"{}\"] {{ content: url(\"{}\"); }}\n", glyph, u).as_bytes(),
            )
            .unwrap();
        }
    }

    let mut file_3d = create_css_file(path, discord_filenames.three_d);
    let mut file_color = create_css_file(path, discord_filenames.color);
    let mut file_flat = create_css_file(path, discord_filenames.flat);
    let mut file_hight_contrast = create_css_file(path, discord_filenames.hc);
    let mut file_animated = create_css_file(path, discord_filenames.animated);

    for _glyph in glyphs {
        let glyph = _glyph.glyph.as_str();
        let url_3d = _glyph.url.three_d.as_ref();
        let url_color = _glyph.url.color.as_ref();
        let url_flat = _glyph.url.flat.as_ref();
        let url_high_contrast = _glyph.url.high_contrast.as_ref();
        let url_animated = _glyph.url.animated.as_ref();

        _write_css(&mut file_3d, glyph, url_3d);
        _write_css(&mut file_color, glyph, url_color);
        _write_css(&mut file_flat, glyph, url_flat);
        _write_css(&mut file_hight_contrast, glyph, url_high_contrast);
        _write_css(&mut file_animated, glyph, url_animated.or(url_3d));
    }

    println!(
        "CSS generated at: {}, {}, {}, {}, {}",
        path.join("fluent_3d.css").display(),
        path.join("fluent_color.css").display(),
        path.join("fluent_flat.css").display(),
        path.join("fluent_high_contrast.css").display(),
        path.join("fluent_animated.css").display()
    );
}
