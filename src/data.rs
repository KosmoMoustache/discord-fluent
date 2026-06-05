use serde::{Deserialize, Serialize};
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
        if let Some((codepoints, rest)) = line.split_once(';') {
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

                let fluentui_emoji = get_fluentui_emoji(&glyph_name, normal_fluent);
                let fluent_ui_animated = get_animated_fluentui_emoji(&glyph_name, animated_fluent);

                if let Some(glyph_name_corr) = fluentui_emoji.glyph_name.as_ref() {
                    let should_insert = if let Some(last) = glyphs.last() {
                        let last_name = &last.glyph_name;
                        let last_status = &last.status;
                        // don't insert if: glyph has the same name as the last and last glyph is fully-qualified
                        !(last_name == &glyph_name && last_status == "fully-qualified")
                    } else {
                        true
                    };

                    if should_insert {
                        let dj = Glyphs {
                            glyph: glyph.to_string(),
                            glyph_name,
                            unicode,
                            status: status.to_string(),
                            path: GlyphURI {
                                three_d: fluentui_emoji.path_3d.clone(),
                                color: fluentui_emoji.path_color.clone(),
                                flat: fluentui_emoji.path_flat.clone(),
                                high_contrast: fluentui_emoji.path_high_contrast.clone(),
                                animated: fluent_ui_animated.path.clone(),
                            },
                            url: GlyphURI {
                                three_d: fluentui_emoji.path_3d.as_ref().and_then(|p| {
                                    format_url_from_path(glyph_name_corr, p)
                                }),
                                color: fluentui_emoji.path_color.as_ref().and_then(|p| {
                                    format_url_from_path(glyph_name_corr, p)
                                }),
                                flat: fluentui_emoji.path_flat.as_ref().and_then(|p| {
                                    format_url_from_path(glyph_name_corr, p)
                                }),
                                high_contrast: fluentui_emoji.path_high_contrast.as_ref().and_then(|p| {
                                    format_url_from_path(glyph_name_corr, p)
                                }),
                                animated: fluent_ui_animated.path.as_ref().and_then(|p| {
                                    format_url_animated_from_path(glyph_name_corr, p)
                                }),
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
        File::create(path).unwrap_or_else(|_| panic!("Failed to create {}", path.display()));
    file.write_all(json.as_bytes())
        .unwrap_or_else(|_| panic!("Failed to write {}", path.display()));
    println!("generated at {:?} with {} glyphs", path, glyphs.len());
}

pub fn format_url_from_path(name: &str, path: &str) -> Option<String> {
    let p = Path::new(path);
    let filename = p.file_name()?.to_str()?;
    let parent = p.parent()?;
    let variant_segment = parent.file_name()?.to_str()?;
    let parent_parent = parent.parent()?;
    let skintone_or_name = parent_parent.file_name()?.to_str()?;

    let path_part = if skintone_or_name == name {
        encode(variant_segment).to_string()
    } else {
        format!("{}/{}", encode(skintone_or_name), encode(variant_segment))
    };

    Some(format!(
        "https://raw.githubusercontent.com/microsoft/fluentui-emoji/refs/heads/main/assets/{}/{}/{}",
        encode(name),
        path_part,
        encode(filename)
    ))
}

pub fn format_url_animated_from_path(name: &str, path: &str) -> Option<String> {
    let p = Path::new(path);
    let filename = p.file_name()?.to_str()?;
    let parent = p.parent()?;
    // parent is "animated"
    let parent_parent = parent.parent()?;
    let skintone_or_name = parent_parent.file_name()?.to_str()?;

    let skintone_path = if skintone_or_name == name {
        "".to_string()
    } else {
        format!("{}/", encode(skintone_or_name))
    };

    Some(format!(
        "https://media.githubusercontent.com/media/microsoft/fluentui-emoji-animated/refs/heads/main/assets/{}/{}animated/{}",
        encode(name),
        skintone_path,
        encode(filename)
    ))
}

fn get_fluentui_emoji(name: &str, glyphs_dir: &Path) -> FluentUi {
    let (p_name, skintone_str) = parse_glyph_name(name);
    let c_name = glyph_name_correction(&p_name);
    let skintone = skintone_str.map(|s| Skintone::from_str(&s));

    let mut result = FluentUi {
        path_3d: None,
        path_color: None,
        path_flat: None,
        path_high_contrast: None,
        skintone,
        glyph_name: Some(c_name.clone()),
    };

    let variants = [
        EmojiVariant::ThreeD,
        EmojiVariant::Color,
        EmojiVariant::Flat,
        EmojiVariant::HighContrast,
    ];

    for variant in variants {
        let (found_path, found_skintone) = find_asset_path(glyphs_dir, &c_name, skintone, variant);
        if let Some(path) = found_path {
            match variant {
                EmojiVariant::ThreeD => result.path_3d = Some(path),
                EmojiVariant::Color => result.path_color = Some(path),
                EmojiVariant::Flat => result.path_flat = Some(path),
                EmojiVariant::HighContrast => result.path_high_contrast = Some(path),
            }
            if result.skintone.is_none() || result.skintone == Some(Skintone::Default) {
                result.skintone = found_skintone;
            }
        }
    }

    result
}

fn get_animated_fluentui_emoji(name: &str, glyphs_dir: &Path) -> FluentUiAnimated {
    let (p_name, skintone_str) = parse_glyph_name(name);
    let c_name = glyph_name_correction(&p_name);
    let skintone = skintone_str.map(|s| Skintone::from_str(&s));

    let (path, _found_skintone) = find_animated_asset_path(glyphs_dir, &c_name, skintone);

    FluentUiAnimated { path }
}

fn find_asset_path(
    glyphs_dir: &Path,
    c_name: &str,
    skintone: Option<Skintone>,
    variant: EmojiVariant,
) -> (Option<String>, Option<Skintone>) {
    let mut check_skintones = match skintone {
        Some(s) if s != Skintone::Default => vec![Some(s), Some(Skintone::Default)],
        _ => vec![Some(Skintone::Default)],
    };
    check_skintones.push(None);

    for s in check_skintones {
        let glyph_path = get_glyph_path(&s, glyphs_dir, c_name);
        // Try both [glyph_path]/[variant] and [glyph_path]/Default/[variant]
        let paths_to_try = vec![
            Path::new(&glyph_path).join(variant.as_path_segment()),
            Path::new(&glyph_path).join("Default").join(variant.as_path_segment()),
        ];

        for variant_path in paths_to_try {
            if variant_path.is_dir() {
                let base_name = c_name.to_lowercase().replace(' ', "_");
                let variant_suffix = variant.as_file_suffix();
                let skintone_suffix = s.map_or("".to_string(), |st| {
                    if st == Skintone::Default {
                        "".to_string()
                    } else {
                        format!("_{}", st.as_file_suffix())
                    }
                });

                let extension = if matches!(variant, EmojiVariant::ThreeD) {
                    "png"
                } else {
                    "svg"
                };
                let filenames = vec![
                    format!("{}_{}{}.{}", base_name, variant_suffix, skintone_suffix, extension),
                    format!("{}_{}.{}", base_name, variant_suffix, extension),
                ];

                for filename in filenames {
                    let full_path = variant_path.join(filename);
                    if full_path.exists() {
                        return (Some(full_path.to_str().unwrap().to_string()), s);
                    }
                }
            }
        }
    }

    (None, None)
}

fn find_animated_asset_path(
    glyphs_dir: &Path,
    c_name: &str,
    skintone: Option<Skintone>,
) -> (Option<String>, Option<Skintone>) {
    let mut check_skintones = match skintone {
        Some(s) if s != Skintone::Default => vec![Some(s), Some(Skintone::Default)],
        _ => vec![Some(Skintone::Default)],
    };
    check_skintones.push(None);

    for s in check_skintones {
        let glyph_path = get_glyph_path(&s, glyphs_dir, c_name);
        // Try multiple path patterns:
        // 1. [glyph_path]/animated
        // 2. [glyph_path]/Default/animated
        // 3. [glyphs_dir]/[c_name]/animated (ignoring skintone in path)
        // 4. [glyphs_dir]/[c_name]/Default/animated
        let paths_to_try = vec![
            Path::new(&glyph_path).join("animated"),
            Path::new(&glyph_path).join("Default").join("animated"),
            glyphs_dir.join(c_name).join("animated"),
            glyphs_dir.join(c_name).join("Default").join("animated"),
        ];

        for animated_path in paths_to_try {
            if animated_path.is_dir() {
                let base_name = c_name.to_lowercase().replace(' ', "_");
                let skintone_suffix = s.map_or("".to_string(), |st| {
                    if st == Skintone::Default {
                        "".to_string()
                    } else {
                        format!("_{}", st.as_file_suffix())
                    }
                });

                let filenames = vec![
                    format!("{}_animated{}.png", base_name, skintone_suffix),
                    format!("{}_animated.png", base_name),
                ];

                for filename in filenames {
                    let full_path = animated_path.join(filename);
                    if full_path.exists() {
                        return (Some(full_path.to_str().unwrap().to_string()), s);
                    }
                }
            }
        }
    }

    (None, None)
}
