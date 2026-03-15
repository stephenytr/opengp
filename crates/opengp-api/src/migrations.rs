use sqlx::PgPool;
use std::path::Path;
use tracing::info;

pub async fn run_migrations(pool: &PgPool) -> Result<(), crate::ApiError> {
    let migrations_dir = Path::new("/app/migrations");

    if !migrations_dir.exists() {
        info!("Migrations directory not found at /app/migrations, skipping");
        return Ok(());
    }

    let mut entries = std::fs::read_dir(migrations_dir)
        .map_err(|e| crate::ApiError::Configuration(e.to_string()))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "sql"))
        .collect::<Vec<_>>();
    
    entries.sort_by_key(|e| e.file_name());

    let mut conn = pool
        .acquire()
        .await
        .map_err(|e| crate::ApiError::Configuration(format!("Failed to acquire migration connection: {e}")))?;

    for entry in entries {
        let path = entry.path();
        let sql = std::fs::read_to_string(&path)
            .map_err(|e| crate::ApiError::Configuration(e.to_string()))?;
        
        if !sql.trim().is_empty() {
            // Execute the entire SQL file as a single statement to preserve DO $$ ... $$; blocks
            // which contain internal semicolons that would break statement splitting
            let result = sqlx::raw_sql(&sql).execute(&mut *conn).await;
            if let Err(e) = result {
                if !e.to_string().contains("duplicate key") {
                    return Err(crate::ApiError::Configuration(format!(
                        "Migration failed: {} - in {}", 
                        e, 
                        path.file_name().unwrap_or_default().to_string_lossy()
                    )));
                }
            }
            info!("Ran migration: {}", path.file_name().unwrap_or_default().to_string_lossy());
        }
    }

    info!("Migrations complete");
    Ok(())
}
