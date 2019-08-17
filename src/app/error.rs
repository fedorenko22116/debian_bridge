#[derive(Debug, Clone)]
pub enum AppError {
    Docker,
    File(String),
    Program(String),
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AppError::Docker => "Cannot connect to docker daemon",
                AppError::File(error) => {
                    Box::leak(format!("IO errors occured: {}", error).into_boxed_str())
                }
                AppError::Program(error) => {
                    Box::leak(format!("Program errors occured: {}", error).into_boxed_str())
                }
            }
        )
    }
}
