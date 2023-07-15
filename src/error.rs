use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum FramerError {
    WinitOsError(winit::error::OsError),
}

impl From<winit::error::OsError> for FramerError {
    fn from(value: winit::error::OsError) -> Self {
        Self::WinitOsError(value)
    }
}

impl Display for FramerError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for FramerError {}
