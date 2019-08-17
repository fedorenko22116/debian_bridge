#[derive(Debug, Clone)]
pub enum SystemError {
    DockerConnection,
}

impl std::error::Error for SystemError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl std::fmt::Display for SystemError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SystemError::DockerConnection => "Cannot connect to docker daemon",
            }
        )
    }
}
