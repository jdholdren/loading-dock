use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;

#[derive(Debug)]
pub struct Error(String);

impl Error {
    fn new(msg: &str) -> Self {
        Error(msg.to_owned())
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Oh no, something bad went down: {}", self.0)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::new(&e.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::new(&e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait WithMessage<T> {
    fn with_context(self, msg: &str) -> Result<T>;
}

impl<T, E: std::error::Error> WithMessage<T> for std::result::Result<T, E> {
    fn with_context(self, msg: &str) -> Result<T> {
        match self {
            Err(err) => Err(Error::new(&format!("{}: {}", msg, err))),
            Ok(val) => Ok(val),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Config {
    pub staged: Vec<String>,
}

pub fn stage_file(cfg: &mut Config, file_name: &str) -> Result<()> {
    // Check that the file exists
    fs::File::open(file_name)?;

    // Dumb check here for a duplicate file
    for staged in &cfg.staged {
        if staged == file_name {
            return Ok(());
        }
    }

    // Alright we're good to stage the file
    cfg.staged.push(file_name.to_owned());

    Ok(())
}

#[cfg(test)]
mod stage_file_tests {
    use super::*;

    #[test]
    fn successfuly_staged() {
        // Create a temp file to stage
        let f = tempfile::NamedTempFile::new().expect("error creating temp file");
        let path = f
            .path()
            .as_os_str()
            .to_str()
            .expect("error converting path to str");

        let mut cfg = Config::default();
        stage_file(&mut cfg, &path).expect("error staging file");

        assert_eq!(cfg.staged, vec![path]);
    }

    #[test]
    fn stage_duplicate() {
        // Create a temp file to stage
        let f = tempfile::NamedTempFile::new().expect("error creating temp file");
        let path = f
            .path()
            .as_os_str()
            .to_str()
            .expect("error converting path to str");

        let mut cfg = Config::default();
        stage_file(&mut cfg, &path).expect("error staging file");
        stage_file(&mut cfg, &path).expect("error staging second file");

        assert_eq!(cfg.staged, vec![path]);
    }

    #[test]
    fn file_does_not_exist() {
        let mut cfg = Config::default();
        stage_file(&mut cfg, "").expect_err("should have errored on missing file");

        assert_eq!(cfg.staged, Vec::<String>::new());
    }
}
