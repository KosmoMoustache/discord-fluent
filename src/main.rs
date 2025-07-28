use lazy_static::lazy_static;
use regex::Regex;
use serde_json::Value;
use std::{
    fs::{self, File},
    io::Write,
};
use urlencoding::encode;
use walkdir::WalkDir;

lazy_static! {
    static ref EMOJIS_PATH: String = "public/fluentui-emoji/assets/".to_owned();
    static ref EMOJI_REGEX: Regex = Regex::new("\"glyph\": \"([^\"]+)\"").unwrap();
    static ref UNICODE_SKINTONES: Regex = Regex::new("\"unicodeSkintones\": \"([^\"]+)\"").unwrap();
}

fn main() {
    let emojis = fs::read_dir(EMOJIS_PATH.as_str()).unwrap();

    let mut css_animated = File::create("public/css/fluent-animated.css").unwrap();
    let mut css_3d = File::create("public/css/fluent-3d.css").unwrap();
    let mut css_color = File::create("public/css/fluent-color.css").unwrap();
    let mut css_flat = File::create("public/css/fluent-flat.css").unwrap();
    let mut css_high_contrast = File::create("public/css/fluent-high-contrast.css").unwrap();

    for emoji in emojis {
        let name: String = emoji.unwrap().file_name().into_string().unwrap();

        let metadata_path = format!("{}/{name}/metadata.json", EMOJIS_PATH.as_str());
        let metadata = fs::read_to_string(metadata_path).unwrap();

        let metadata_json: Value = serde_json::from_str(&metadata).unwrap();
        let glyph_emoji = metadata_json.get("glyph").unwrap().as_str().unwrap();

        let unicode_skintones = metadata_json
            .get("unicodeSkintones")
            .and_then(|unicode_skintones| unicode_skintones.as_array())
            .map(|array| {
                array
                    .iter()
                    .map(|skintone| {
                        skintone
                            .as_str()
                            .unwrap()
                            .split(' ')
                            .map(|s| s.to_string())
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>()
            });

        let is_skintone_emoji = metadata.contains("Skintones");
        let skinetones = get_skintones(&name);

        if is_skintone_emoji {
            if let Some(unicode_skintones) = &unicode_skintones {
                let mut i = 0;
                while i < unicode_skintones.len() {
                    let skintone = &skinetones[unicode_skintones.len() - i - 1];
                    write_css(
                        &mut css_animated,
                        &mut css_3d,
                        &mut css_color,
                        &mut css_flat,
                        &mut css_high_contrast,
                        &emoji_from_codepoints(&unicode_skintones[i]),
                        &name,
                        Some(skintone.as_str()),
                    );
                    i += 1;
                }
            }
        } else {
            write_css(
                &mut css_animated,
                &mut css_3d,
                &mut css_color,
                &mut css_flat,
                &mut css_high_contrast,
                glyph_emoji,
                &name,
                None,
            );
        }

        println!("Generated css for {glyph_emoji}! with {skinetones:?}");
    }

    css_animated.flush().unwrap();
    css_3d.flush().unwrap();
    css_color.flush().unwrap();
    css_flat.flush().unwrap();
    css_high_contrast.flush().unwrap();

    println!("Done!");
}

fn write_css(
    css_animated: &mut File,
    css_3d: &mut File,
    css_color: &mut File,
    css_flat: &mut File,
    css_high_contrast: &mut File,
    emoji: &str,
    name: &str,
    skintone_variant: Option<&str>,
) {
    if let Some(css) = get_animated_css(emoji, &name, &skintone_variant) {
        css_animated.write_all(css.as_bytes()).unwrap();
    } else {
        css_animated
            .write_all(get_css(emoji, &name, &skintone_variant, "3D").as_bytes())
            .unwrap();
    };

    css_3d
        .write_all(get_css(emoji, &name, &skintone_variant, "3D").as_bytes())
        .unwrap();
    css_color
        .write_all(get_css(emoji, &name, &skintone_variant, "Color").as_bytes())
        .unwrap();
    css_flat
        .write_all(get_css(emoji, &name, &skintone_variant, "Flat").as_bytes())
        .unwrap();
    css_high_contrast
        .write_all(get_css(emoji, &name, &skintone_variant, "High Contrast").as_bytes())
        .unwrap();
}

fn get_skintones(name: &str) -> Vec<String> {
    let skintone_path = fs::read_dir(format!("{}/{name}", EMOJIS_PATH.as_str())).unwrap();
    let mut unicodes = Vec::new();

    for skintone in skintone_path {
        let tone = skintone.unwrap().file_name().into_string().unwrap();
        if tone == "metadata.json" {
            continue;
        }

        unicodes.push(tone);
    }
    return unicodes;
}

fn get_css(emoji: &str, name: &str, skintone_variant: &Option<&str>, variant: &str) -> String {
    let p_variant = variant.to_lowercase().replace(" ", "_");

    let url: String;
    if let Some(skintone) = &skintone_variant {
        url = format!(
            "https://raw.githubusercontent.com/microsoft/fluentui-emoji/refs/heads/main/assets/{}/{}/{}_{}.{}",
            encode(name),
            format!("{}/{}", skintone, encode(variant)),
            name.to_lowercase().replace(" ", "_"),
            format!("{}_{}", p_variant, skintone.to_lowercase()),
            if variant == "3D" { "png" } else { "svg" }
        );
    } else {
        url = format!(
                "https://raw.githubusercontent.com/microsoft/fluentui-emoji/refs/heads/main/assets/{}/{}/{}_{}.{}",
                encode(name),
                encode(variant).to_string()
                ,
                name.to_lowercase().replace(" ", "_"),
                    p_variant
                ,
                if variant == "3D" { "png" } else { "svg" }
            );
    }

    format!("img[alt|=\"{emoji}\"] {{ content: url(\"{}\"); }}\n", url)
}

fn get_animated_css(emoji: &str, name: &str, skintone_variant: &Option<&str>) -> Option<String> {
    let l_name = name.to_lowercase();

    let primary = find_file_in_dir("public/animated-fluent-emoji/Emojis", |entry| {
        if let Some(skintone) = &skintone_variant {
            if skintone.to_lowercase() != "default" {
                entry.file_name().to_str().unwrap().to_lowercase().trim()
                    == format!("{} {} skin tone.png", l_name, skintone.to_lowercase())
            } else {
                entry.file_name().to_str().unwrap().to_lowercase().trim()
                    == format!("{}.png", l_name)
            }
        } else {
            entry.file_name().to_str().unwrap().to_lowercase().trim() == format!("{}.png", l_name)
        }
    });

    let mut url: Option<String> = None;

    if let Some(primary_path) = primary {
        url = Some(format!(
            "https://discord-fluent.siris.me/{}",
            encode(&primary_path)
        ));
    } else {
        // fallback to fluentui-emoji-animated (media.githubusercontent.com)
        let secondary_dir_path = if let Some(skintone) = &skintone_variant {
            format!(
                "public/fluentui-emoji-animated/assets/{}/{}/animated",
                name, skintone
            )
        } else {
            format!("public/fluentui-emoji-animated/assets/{}/animated", name)
        };

        let secondary = find_file_in_dir(&secondary_dir_path, |entry| {
            if let Some(skintone) = &skintone_variant {
                entry.file_name().to_str().unwrap().to_lowercase()
                    == format!(
                        "{}_animated_{}.png",
                        l_name.replace(" ", "_"),
                        skintone.to_lowercase()
                    )
            } else {
                entry.file_name().to_str().unwrap().to_lowercase()
                    == format!("{}_animated.png", l_name.replace(" ", "_"))
            }
        });

        if let Some(secondary_path) = secondary {
            url = Some(format!(
                "https://media.githubusercontent.com/media/microsoft/fluentui-emoji-animated/main/assets/{}",
                encode(&secondary_path.replace("fluentui-emoji-animated/assets/", ""))
            ));
        }
    }

    if let Some(url) = url {
        Some(format!(
            "img[alt|=\"{emoji}\"] {{ content: url(\"{}\"); }}\n",
            url
        ))
    } else {
        eprintln!("Animated Emoji not found for {name}");
        None
    }
}

fn find_file_in_dir(dir: &str, predicate: impl Fn(&walkdir::DirEntry) -> bool) -> Option<String> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .find(predicate)
        .map(|entry| entry.path().display().to_string().replace("public/", ""))
}

fn emoji_from_codepoints(codepoint_pair: &[String]) -> String {
    codepoint_pair
        .iter()
        .flat_map(|hex| {
            u32::from_str_radix(hex, 16)
                .ok()
                .and_then(std::char::from_u32)
        })
        .collect()
}
