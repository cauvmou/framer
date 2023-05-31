use std::{collections::HashMap, path::PathBuf, sync::RwLock};

use lazy_static::lazy_static;

pub mod renderer;
pub mod window;

lazy_static! {
    pub static ref FONT_RESOURCES: RwLock<HashMap<String, FontResource>> = {
        let mut map = HashMap::new();
        map.insert(
            "Roboto-Regular".to_string(),
            FontResource {
                path: "./fonts/Roboto/Roboto-Regular.ttf".into(),
                bytes: include_bytes!("./fonts/Roboto/Roboto-Regular.ttf"),
            },
        );
        map.insert(
            "Roboto-Black".to_string(),
            FontResource {
                path: "./fonts/Roboto/Roboto-Black.ttf".into(),
                bytes: include_bytes!("./fonts/Roboto/Roboto-Black.ttf"),
            },
        );
        map.insert(
            "Alegreya-Sans".to_string(),
            FontResource {
                path: "./fonts/AlegreyaSans/AlegreyaSans-Regular.ttf".into(),
                bytes: include_bytes!("./fonts/AlegreyaSans/AlegreyaSans-Regular.ttf"),
            },
        );
        map.insert(
            "SourceCodePro-Regular".to_string(),
            FontResource {
                path: "./fonts/Source_Code_Pro/static/SourceCodePro-Regular.ttf".into(),
                bytes: include_bytes!("./fonts/Source_Code_Pro/static/SourceCodePro-Regular.ttf"),
            },
        );
        RwLock::new(map)
    };
}

#[derive(Debug, Clone, Default)]
pub struct FontResource {
    #[allow(dead_code)]
    path: PathBuf,
    bytes: &'static [u8],
}

// TODO: Rework the font attribute to be a reference to a font resource of some sort idfk
#[derive(Debug, Clone)]
pub struct Text {
    font: String,
    literal: String,
}

impl Text {
    pub fn new(literal: &str) -> Self {
        Self {
            font: "Roboto-Regular".into(),
            literal: literal.to_owned(),
        }
    }
}
