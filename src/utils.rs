use std::{fs, path::Path};

#[derive(Debug)]
pub struct FluentUi {
    pub path_3d: Option<String>,
    pub path_color: Option<String>,
    pub path_flat: Option<String>,
    pub path_high_contrast: Option<String>,
    pub skintone: Option<Skintone>,
    pub glyph_name: Option<String>,
}

#[derive(Debug)]
pub struct FluentUiAnimated {
    pub path: Option<String>,
}

// emoji name fixes: looked up value -> corrected value
pub static GLYPH_NAME_FIXES: &[(&str, &str)] = &[
    ("smiling face with open hands", "Hugging face"),
    ("face with crossed-out eyes", "Face with spiral eyes"),
    ("enraged face", "Pouting face"),
    ("deaf person", "Person deaf"),
    ("deaf woman", "Woman deaf"),
    ("deaf man", "Man deaf"),
    ("breast-feeding", "Breast feeding"),
    ("Mrs. Claus", "Mrs claus"),
    ("superhero", "Person superhero"),
    ("supervillain", "Person supervillain"),
    ("mage", "Person mage"),
    ("fairy", "Person fairy"),
    ("vampire", "Person vampire"),
    ("merperson", "Person merpeople"),
    ("merman", "Man merpeople"),
    ("mermaid", "Woman merpeople"),
    ("elf", "Person elf"),
    ("genie", "Person genie"),
    ("zombie", "Person genie"),
    ("people with bunny ears", "Person with bunny ears"),
    ("men with bunny ears", "Man with bunny ears"),
    ("people wrestling", "Person wrestling"),
    ("men wrestling", "Man wrestling"),
    ("women wrestling", "Woman wrestling"),
    ("women with bunny ears", "Woman with bunny ears"),
    ("black bird", "Blackbird"),
    ("phoenix", "Phoenix bird"),
    ("keycap: #", "Keycap hashtag"),
    ("keycap: *", "Keycap asterisk"),
    ("keycap: 0", "Keycap 0"),
    ("keycap: 1", "Keycap 1"),
    ("keycap: 2", "Keycap 2"),
    ("keycap: 3", "Keycap 3"),
    ("keycap: 4", "Keycap 4"),
    ("keycap: 5", "Keycap 5"),
    ("keycap: 6", "Keycap 6"),
    ("keycap: 7", "Keycap 7"),
    ("keycap: 8", "Keycap 8"),
    ("keycap: 9", "Keycap 9"),
    ("keycap: 10", "Keycap 10"),
    ("A button (blood type)", "A button blood type"),
    ("AB button (blood type)", "Ab button blood type"),
    ("B button (blood type)", "B button blood type"),
    ("O button (blood type)", "O button blood type"),
    ("Red triangle pointed up", "Red triangle"),
    ("Piata", "Piñata"),
];
pub fn glyph_name_correction(name: &str) -> String {
    GLYPH_NAME_FIXES
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.to_string())
        .unwrap_or_else(|| name.to_string())
}

#[derive(Clone, Copy, Debug)]
pub enum EmojiVariant {
    ThreeD,
    Color,
    Flat,
    HighContrast,
}

impl EmojiVariant {
    pub fn as_str(&self) -> &'static str {
        match self {
            EmojiVariant::ThreeD => "3D",
            EmojiVariant::Color => "Color",
            EmojiVariant::Flat => "Flat",
            EmojiVariant::HighContrast => "High Contrast",
        }
    }

    pub fn as_path_segment(&self) -> &'static str {
        self.as_str()
    }

    pub fn as_file_suffix(&self) -> &'static str {
        match self {
            EmojiVariant::ThreeD => "3d",
            EmojiVariant::Color => "color",
            EmojiVariant::Flat => "flat",
            EmojiVariant::HighContrast => "high_contrast",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Skintone {
    Default,
    Light,
    MediumLight,
    Medium,
    MediumDark,
    Dark,
}

impl Skintone {
    pub fn as_str(&self) -> &'static str {
        match self {
            Skintone::Default => "Default",
            Skintone::Light => "Light",
            Skintone::MediumLight => "Medium-Light",
            Skintone::Medium => "Medium",
            Skintone::MediumDark => "Medium-Dark",
            Skintone::Dark => "Dark",
        }
    }

    pub fn as_path_segment(&self) -> &'static str {
        self.as_str()
    }

    pub fn as_file_suffix(&self) -> String {
        match self {
            Skintone::Default => "".to_string(),
            _ => self.as_str().to_lowercase(),
        }
    }

    pub fn from_str(skintone: &str) -> Skintone {
        match skintone.to_ascii_lowercase().as_str() {
            "light skin tone" => Skintone::Light,
            "medium-light skin tone" => Skintone::MediumLight,
            "medium skin tone" => Skintone::Medium,
            "medium-dark skin tone" => Skintone::MediumDark,
            "dark skin tone" => Skintone::Dark,
            "default" => Skintone::Default,
            _ => Skintone::Default,
        }
    }
}

pub fn path_exist(path: &str) -> Result<String, String> {
    if fs::metadata(path).is_ok() {
        Ok(path.to_string())
    } else {
        Err(format!("Path '{}' does not exist", path))
    }
}

pub fn parse_glyph_name(name: &str) -> (String, Option<String>) {
    // Split by colon (used for skintones)
    let (name_part, skintone) = if let Some((before_colon, after_colon)) = name.split_once(':') {
        // there is a space after colon (mainly an exection for keycap: [0..10]|*|#)
        if after_colon.split_whitespace().count() >= 3 {
            (before_colon.trim(), Some(after_colon.trim().to_string()))
        } else {
            (name.trim(), None)
        }
    } else {
        (name.trim(), None)
    };

    // Make the first character uppercase and the rest lowercase
    let name_corr = name_part.chars().next().map_or(String::new(), |f| {
        f.to_uppercase().collect::<String>() + &name_part[1..].to_lowercase()
    });

    (name_corr, skintone)
}

pub fn get_glyph_path(skintone: &Option<Skintone>, glyphs_dir: &Path, name: &str) -> String {
    let mut path = glyphs_dir.to_path_buf();
    path.push(name);
    if let Some(s) = skintone {
        if *s != Skintone::Default {
            path.push(s.as_path_segment());
        }
    }
    path.to_str().unwrap().to_string()
}
