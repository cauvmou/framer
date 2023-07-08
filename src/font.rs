use std::path::PathBuf;

#[derive(Debug, Clone, Default, Hash, Eq, PartialEq)]
pub struct Font {
    pub family: Family,
    pub weight: Weight,
    pub monospace: bool,
    pub src: Source,
    pub data: &'static [u8],
}

impl Font {
    pub const DEFAULT: Font = Font {
        family: Family::SansSerif,
        weight: Weight::Medium,
        monospace: false,
        src: Source::Builtin,
        data: include_bytes!("./fonts/DejaVuSerif.ttf"),
    };

    pub const MONOSPACE: Font = Font {
        family: Family::Monospace,
        weight: Weight::Medium,
        monospace: true,
        src: Source::Builtin,
        data: include_bytes!("./fonts/Source_Code_Pro/static/SourceCodePro-Medium.ttf"),
    };

    pub const CAIRO: Font = Font {
        family: Family::Named("Cairo"),
        weight: Weight::Medium,
        monospace: false,
        src: Source::Builtin,
        data: include_bytes!("./fonts/Cairo/static/Cairo-Regular.ttf"),
    };
}

#[derive(Debug, Clone, Default, Hash, Eq, PartialEq)]
pub enum Source {
    #[default]
    Builtin,
    Path(PathBuf),
    Url(&'static str),
}

#[derive(Debug, Clone, Copy, Default, Hash, Eq, PartialEq)]
pub enum Family {
    Named(&'static str),
    Serif,
    #[default]
    SansSerif,
    Cursive,
    Monospace,
}

#[derive(Debug, Clone, Copy, Default, Hash, Eq, PartialEq)]
pub enum Weight {
    Thin = 100,
    ExtraLigh = 200,
    Light = 300,
    #[default]
    Normal = 400,
    Medium = 500,
    Semibold = 600,
    Bold = 700,
    ExtraBold = 800,
    Black = 900,
}
