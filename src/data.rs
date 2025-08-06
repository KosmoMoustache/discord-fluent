use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use urlencoding::encode;

use crate::utils::{
    get_glyph_path, glyph_name_correction, parse_glyph_name, EmojiVariant, FluentUi,
    FluentUiAnimated, Skintone,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct GlyphURI {
    pub three_d: Option<String>,
    pub color: Option<String>,
    pub flat: Option<String>,
    pub high_contrast: Option<String>,
    pub animated: Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Glyphs {
    pub glyph: String,
    pub glyph_name: String,
    pub unicode: String,
    pub status: String,
    pub path: GlyphURI,
    pub url: GlyphURI,
}

pub fn create_glyph_data(
    path: &Path,
    normal_fluent: &Path,
    animated_fluent: &Path,
    verbose_duplication: bool,
) {
    let url = "https://www.unicode.org/Public/emoji/latest/emoji-test.txt";
    let mut resp = reqwest::blocking::get(url).expect("Failed to download emoji-test.txt");
    let mut content = String::new();
    resp.read_to_string(&mut content)
        .expect("Failed to read content");

    let mut glyphs: Vec<Glyphs> = Vec::new();
    // Array of glyphs names to skip
    let skip_glyphs = [
        "face with bags under eyes", // E16.0
        "fingerprint",               // E16.0
        "leafless tree",             // E16.0
        "root vegetable",            // E16.0
        "harp",                      // E16.0
        "shovel",                    // E16.0
        "splatter",                  // E16.0
        "light skin tone",           // Component
        "medium skin tone",          // Component
        "medium-light skin tone",    // Component
        "medium-dark skin tone",     // Component
        "dark skin tone",            // Component
        "red hair",                  // Component
        "curly hair",                // Component
        "white hair",                // Component
        "bald",                      // Component
    ];
    for line in content.lines() {
        // Skip comments and empty lines
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        // Example line: 1F3C3 1F3FB ; fully-qualified # 🏃🏻 E1.0 person running: light skin tone
        if let Some((codepoints, rest)) = line.split_once(";") {
            let codepoints = codepoints.trim();
            if let Some((status, after_hash)) = rest.split_once('#') {
                let status = status.trim(); // e.g. "fully-qualified"
                let after_hash = after_hash.trim();
                // after_hash: 😋 E0.6 face savoring food
                let mut parts = after_hash.split_whitespace();
                let glyph = parts.next().unwrap_or("");
                let _emoji_version = parts.next(); // E0.6
                let glyph_name = parts.collect::<Vec<_>>().join(" ");

                // Skip glyphs
                if skip_glyphs.contains(&glyph_name.as_str()) || glyph_name.contains("flag:") {
                    continue;
                }

                let unicode = codepoints.split_whitespace().collect::<Vec<_>>().join(" ");

                // ! DEBUG
                // if !glyph_name.contains("waving") && !glyph_name.contains("triangle") {
                //     continue;
                // }

                let fluentui_emoji = get_fluentui_emoji(&glyph_name, normal_fluent);
                let fluent_ui_animated = get_animated_fluentui_emoji(&glyph_name, animated_fluent);

                if let Some(glyph_name_corr) = fluentui_emoji.glyph_name {
                    let should_insert = if let Some(last) = glyphs.last() {
                        let last_name = last.glyph_name.clone();
                        let last_status = last.status.clone();
                        // don't insert if: glyph has the same name as the last and last glyph is fully-qualified
                        !(last_name == glyph_name && last_status == "fully-qualified")
                    } else {
                        true
                    };

                    if should_insert {
                        let dj = Glyphs {
                            glyph: glyph.to_string(),
                            glyph_name: glyph_name,
                            unicode: unicode,
                            status: status.to_string(),
                            path: GlyphURI {
                                three_d: fluentui_emoji.path_3d.clone(),
                                color: fluentui_emoji.path_color.clone(),
                                flat: fluentui_emoji.path_flat.clone(),
                                high_contrast: fluentui_emoji.path_high_contrast.clone(),
                                animated: fluent_ui_animated.path.clone(),
                            },
                            url: GlyphURI {
                                three_d: if fluentui_emoji.path_3d.is_some() {
                                    Some(format_url(
                                        glyph_name_corr.as_ref(),
                                        EmojiVariant::ThreeD,
                                        fluentui_emoji.skintone.as_ref(),
                                    ))
                                } else {
                                    None
                                },
                                color: if fluentui_emoji.path_color.is_some() {
                                    Some(format_url(
                                        glyph_name_corr.as_ref(),
                                        EmojiVariant::Color,
                                        fluentui_emoji.skintone.as_ref(),
                                    ))
                                } else {
                                    None
                                },
                                flat: if fluentui_emoji.path_flat.is_some() {
                                    Some(format_url(
                                        glyph_name_corr.as_ref(),
                                        EmojiVariant::Flat,
                                        fluentui_emoji.skintone.as_ref(),
                                    ))
                                } else {
                                    None
                                },
                                high_contrast: if fluentui_emoji.path_high_contrast.is_some() {
                                    Some(format_url(
                                        glyph_name_corr.as_ref(),
                                        EmojiVariant::HighContrast,
                                        fluentui_emoji.skintone.as_ref(),
                                    ))
                                } else {
                                    None
                                },
                                animated: if fluent_ui_animated.path.is_some() {
                                    Some(format_url_animated(
                                        fluent_ui_animated.glyph_name.unwrap().as_ref(),
                                        fluentui_emoji.skintone.as_ref(),
                                    ))
                                } else {
                                    None
                                },
                            },
                        };
                        glyphs.push(dj)
                    } else {
                        if verbose_duplication {
                            println!(
                                "Skipping duplicate glyph: {} ({}) (duplicate:{})",
                                glyph_name_corr,
                                glyph,
                                glyphs.last().unwrap().status
                            );
                        }
                    }
                }
            }
        }
    }
    let json = serde_json::to_string_pretty(&glyphs).unwrap();
    let mut file =
        File::create(&path).expect(format!("Failed to create {}", path.display()).as_ref());
    file.write_all(json.as_bytes())
        .expect(format!("Failed to wirte {}", path.display()).as_ref());
    println!("generated at {:?} with {} glyphs", path, glyphs.len());
}

pub fn format_url(name: &str, variant: EmojiVariant, skintone: Option<&Skintone>) -> String {
    return format!(
        //   https://raw.githubusercontent.com/microsoft/fluentui-emoji/refs/heads/main/assets/{name}/{skintone/variant}/{name}_{variant/skintone}.{ext}",
            "https://raw.githubusercontent.com/microsoft/fluentui-emoji/refs/heads/main/assets/{}/{}/{}_{}.{}",
            encode(name),
            skintone.as_ref().map_or(format!("{}", encode(variant.as_str())), |s| format!("{}/{}", s.as_str(), encode(variant.as_str()))),
            name.to_lowercase().replace(" ", "_"),
            skintone.as_ref().map_or(
                format!("{}", encode(variant.as_snake_case().to_lowercase().as_ref())),
                |skintone| format!("{}_{}", encode(variant.as_snake_case().to_lowercase().as_ref()), skintone.as_str().to_lowercase())
            ),
            if matches!(variant, EmojiVariant::ThreeD) { "png" } else { "svg" }
        );
}
pub fn format_url_animated(name: &str, skintone: Option<&Skintone>) -> String {
    return format!(
    //   https://media.githubusercontent.com/media/microsoft/fluentui-emoji-animated/refs/heads/main/assets/{name}{skintone}/animated/{name}_animated{_skintone}.png
        "https://media.githubusercontent.com/media/microsoft/fluentui-emoji-animated/refs/heads/main/assets/{}{}/animated/{}_animated{}.png",
        encode(name),
        skintone.as_ref().map_or("".to_string(), |s| format!("/{}", s.as_str())),
        name.to_lowercase().replace(" ", "_"),
        skintone.as_ref().map_or("".to_string(), |s| format!("_{}", s.as_str().to_lowercase())),
    );
}

fn get_fluentui_emoji(name: &str, glyphs_dir: &Path) -> FluentUi {
    fn _format_filename(
        variant: EmojiVariant,
        path: &String,
        name: &str,
        skintone: Option<&Skintone>,
    ) -> Option<String> {
        return Some(format!(
            "{}/{}_{}{}.{}",
            path,
            name.to_lowercase().replace(" ", "_"),
            variant.as_str().to_lowercase(),
            if skintone.is_some() {
                format!("_{}", skintone.unwrap().as_str().to_lowercase())
            } else {
                "".to_string()
            },
            if matches!(variant, EmojiVariant::ThreeD) {
                "png"
            } else {
                "svg"
            }
        ));
    }
    pub fn _update_variant_path(t: &&str, result: &mut FluentUi, path: &String, name: &str) {
        match *t {
            "3D" => {
                result.path_3d =
                    _format_filename(EmojiVariant::ThreeD, &path, &name, result.skintone.as_ref());
            }
            "Color" => {
                result.path_color =
                    _format_filename(EmojiVariant::Color, &path, &name, result.skintone.as_ref());
            }
            "Flat" => {
                result.path_flat =
                    _format_filename(EmojiVariant::Flat, &path, &name, result.skintone.as_ref());
            }
            "High Contrast" => {
                result.path_high_contrast = _format_filename(
                    EmojiVariant::HighContrast,
                    &path,
                    &name,
                    result.skintone.as_ref(),
                );
            }
            _ => {}
        }
    }

    let (p_name, skintone) = parse_glyph_name(name);
    let c_name = glyph_name_correction(&p_name);
    let skintone: Option<Skintone> = skintone
        .as_ref()
        .map(|s| Some(Skintone::from_str(s)))
        .unwrap_or(None);
    let glyph_path = get_glyph_path(&skintone, &glyphs_dir, &c_name);

    let mut result = FluentUi {
        path_3d: None,
        path_color: None,
        path_flat: None,
        path_high_contrast: None,
        skintone: skintone,
        glyph_name: Some(c_name.to_string()),
    };

    let variants = ["3D", "Color", "Flat", "High Contrast"];
    for variant in &variants {
        let type_path = format!("{}/{}", glyph_path, variant);
        if fs::metadata(&type_path).is_ok() {
            _update_variant_path(variant, &mut result, &type_path, name);
        } else {
            // Default skintone
            let default_path = format!("{}/Default/{}", glyph_path, variant);
            if fs::metadata(&default_path).is_ok() {
                result.skintone = Some(Skintone::Default);
                _update_variant_path(variant, &mut result, &default_path, name);
            }
        }
    }
    result
}

fn get_animated_fluentui_emoji(name: &str, glyphs_dir: &Path) -> FluentUiAnimated {
    let (p_name, skintone) = parse_glyph_name(name);
    let c_name = glyph_name_correction(&p_name);
    let skintone: Option<Skintone> = skintone
        .as_ref()
        .map(|s| Some(Skintone::from_str(s)))
        .unwrap_or(None);
    let glyph_path = get_glyph_path(&skintone, &glyphs_dir, &c_name);

    let mut result: FluentUiAnimated = FluentUiAnimated {
        path: None,
        skintone: skintone,
        glyph_name: Some(c_name.to_string()),
    };

    let check_types = ["animated"];
    for t in &check_types {
        let type_path = format!("{}/{}", glyph_path, t);
        if fs::metadata(&type_path).is_ok() {
            match *t {
                "animated" => {
                    result.path = Some(format!(
                        "{}/{}_animated.png",
                        type_path.clone(),
                        c_name.to_lowercase().replace(" ", "_")
                    ))
                }
                _ => {}
            }
        } else {
            // get default skintone
            let default_path = format!("{}/Default/{}", glyph_path, t);
            if fs::metadata(&default_path).is_ok() {
                result.skintone = Some(Skintone::Default);
                match *t {
                    "animated" => {
                        result.path = Some(format!(
                            "{}/{}_animated.png",
                            type_path.clone(),
                            c_name.to_lowercase().replace(" ", "_")
                        ))
                    }
                    _ => {}
                }
            }
        }
    }
    result
}
