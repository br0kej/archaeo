use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("FailedProcessingError: Failed to process: {0}")]
    FailedProcessing(String),

    #[error("Failed to guess programming lang of {0}")]
    FailedGuessLang(String),

    #[error("Failed to create JSON: {0}")]
    SerdeError(serde_json::Error),

    #[error("Failed to create file: {0}")]
    FileCreationError(std::io::Error),

    #[error("Failed to create CSV: {0}")]
    CSVError(csv::Error),

    #[error(transparent)]
    Other(#[from] color_eyre::Report),
}

impl From<serde_json::Error> for CliError {
    fn from(err: serde_json::Error) -> CliError {
        CliError::SerdeError(err)
    }
}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> CliError {
        CliError::FileCreationError(err)
    }
}

impl From<csv::Error> for CliError {
    fn from(err: csv::Error) -> CliError {
        CliError::CSVError(err)
    }
}
