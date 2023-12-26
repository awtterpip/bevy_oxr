use std::fmt::Display;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, XrError>;

#[derive(Error, Debug)]
pub enum XrError {
    Placeholder,
}

impl Display for XrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
