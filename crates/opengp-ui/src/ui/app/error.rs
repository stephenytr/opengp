/// App-level error type for rat-salsa integration.
/// Uses a boxed trait object for flexibility across different error types.
pub type AppError = Box<dyn std::error::Error + Send + Sync>;
