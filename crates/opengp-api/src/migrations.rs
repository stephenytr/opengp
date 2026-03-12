use sqlx::PgPool;
use std::path::Path;
use tracing::info;

pub async fn run_migrations(pool: &PgPool) -> Result<(), crate::ApiError> {
    let migrations_dir = Path::new("/app/migrations_postgres");

    if !migrations_dir.exists() {
        info!("Migrations directory not found at /app/migrations_postgres, skipping");
        return Ok(());
    }

    let mut entries = std::fs::read_dir(migrations_dir)
        .map_err(|e| crate::ApiError::Configuration(e.to_string()))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "sql"))
        .collect::<Vec<_>>();
    
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let sql = std::fs::read_to_string(&path)
            .map_err(|e| crate::ApiError::Configuration(e.to_string()))?;
        
        if !sql.trim().is_empty() {
            for statement in sql.split(';') {
                let stmt = statement.trim();
                if !stmt.is_empty() && !stmt.starts_with("--") {
                    let result = sqlx::query(stmt).execute(pool).await;
                    if let Err(e) = result {
                        if !e.to_string().contains("duplicate key") {
                            return Err(crate::ApiError::Configuration(format!(
                                "Migration failed: {} - in {}", 
                                e, 
                                path.file_name().unwrap_or_default().to_string_lossy()
                            )));
                        }
                    }
                }
            }
            info!("Ran migration: {}", path.file_name().unwrap_or_default().to_string_lossy());
        }
    }

    info!("Migrations complete");
    Ok(())
}
