use std::{path::Path, str::FromStr, time::Duration};

use sqlx::{
    SqlitePool,
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
};

/// Migrations in `apps/server/migrations`, embedded into the binary at compile time.
static MIGRATOR: Migrator = sqlx::migrate!();

/// Opens (creating if needed) the SQLite database and applies pending migrations.
/// `:memory:` opens a private in-memory database, as the tests do.
pub async fn connect(path: &Path) -> Result<SqlitePool, sqlx::Error> {
    let pool = if path == Path::new(":memory:") {
        // One connection, never recycled: the database lives exactly as long as it does.
        let options = SqliteConnectOptions::from_str(":memory:")?.foreign_keys(true);
        SqlitePoolOptions::new()
            .max_connections(1)
            .idle_timeout(None)
            .max_lifetime(None)
            .connect_with(options)
            .await?
    } else {
        if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir)?;
        }
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .foreign_keys(true)
            .busy_timeout(Duration::from_secs(5));
        SqlitePoolOptions::new().connect_with(options).await?
    };
    MIGRATOR.run(&pool).await?;
    Ok(pool)
}
