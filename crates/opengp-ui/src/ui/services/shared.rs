pub type UiResult<T> = Result<T, UiServiceError>;

#[derive(Debug)]
pub enum UiServiceError {
    NotFound(String),
    Validation(String),
    Repository(String),
    Unknown(String),
}

impl std::fmt::Display for UiServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UiServiceError::NotFound(msg) => write!(f, "Not found: {}", msg),
            UiServiceError::Validation(msg) => write!(f, "Validation error: {}", msg),
            UiServiceError::Repository(msg) => write!(f, "Repository error: {}", msg),
            UiServiceError::Unknown(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for UiServiceError {}

pub trait ToUiError {
    fn to_ui_error(self) -> UiServiceError;
    fn to_ui_repository_error(self) -> UiServiceError;
}

impl<E> ToUiError for E
where
    E: std::error::Error,
{
    fn to_ui_error(self) -> UiServiceError {
        UiServiceError::Unknown(self.to_string())
    }

    fn to_ui_repository_error(self) -> UiServiceError {
        UiServiceError::Repository(self.to_string())
    }
}

/// Extension trait for Result types to convert errors to UiServiceError
pub trait UiResultExt<T> {
    /// Convert repository error to UiServiceError
    fn map_ui_repo_err(self) -> Result<T, UiServiceError>;
    /// Convert generic error to UiServiceError
    fn map_ui_err(self) -> Result<T, UiServiceError>;
}

impl<T, E: std::error::Error> UiResultExt<T> for Result<T, E> {
    fn map_ui_repo_err(self) -> Result<T, UiServiceError> {
        self.map_err(|e| e.to_ui_repository_error())
    }

    fn map_ui_err(self) -> Result<T, UiServiceError> {
        self.map_err(|e| e.to_ui_error())
    }
}
