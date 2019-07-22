use std::path::Path;
use super::program::Program;
use std::error::Error;

#[derive(Clone)]
pub struct Config {
    programs: Vec<Program>,
}

impl Config {
    pub fn new(programs: &Vec<Program>) -> Self {
        Config { programs: programs.to_owned() }
    }

    pub fn deserialize(path: &Path) -> Result<Self, Box<dyn Error>> {
        unimplemented!()
    }

    pub fn serialize(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn should_serialize_success() {
        assert!(true)
    }

    #[test]
    fn should_serialize_failed() {
        assert!(true)
    }

    #[test]
    fn should_deserialize_success() {
        assert!(true)
    }

    #[test]
    fn should_deserialize_failed() {
        assert!(true)
    }
}
