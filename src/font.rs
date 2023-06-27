use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
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
        data: include_bytes!("./fonts/Roboto/Roboto-Regular.ttf"),
    };

    pub const MONOSPACE: Font = Font {
        family: Family::Monospace,
        weight: Weight::Medium,
        monospace: true,
        src: Source::Builtin,
        data: include_bytes!("./fonts/Source_Code_Pro/static/SourceCodePro-Medium.ttf"),
    };
}

#[derive(Debug, Clone, Default)]
pub enum Source {
    #[default]
    Builtin,
    Path (PathBuf),
    Url (&'static str)
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Family {
    Named (&'static str),
    Serif,
    #[default]
    SansSerif,
    Cursive,
    Monospace,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Weight {
    Thin        = 100,
    ExtraLigh   = 200,
    Light       = 300,
    #[default]
    Normal      = 400,
    Medium      = 500,
    Semibold    = 600,
    Bold        = 700,
    ExtraBold   = 800,
    Black       = 900,
}