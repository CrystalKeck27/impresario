use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use std::sync::OnceLock;

use crate::schema::{acoustid_fingerprints, impresario_files};

type DatabasePool = Pool<ConnectionManager<PgConnection>>;

static DATABASE_POOL: OnceLock<DatabasePool> = OnceLock::new();

fn pool() -> Result<&'static DatabasePool, String> {
    if let Some(pool) = DATABASE_POOL.get() {
        return Ok(pool);
    }

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL is not configured on the server".to_string())?;
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .max_size(8)
        .build(manager)
        .map_err(|error| format!("Could not connect to PostgreSQL: {error}"))?;

    // Another request may initialize the pool at the same time; in that case,
    // use the pool that won the race.
    let _ = DATABASE_POOL.set(pool);
    DATABASE_POOL
        .get()
        .ok_or_else(|| "Could not initialize the PostgreSQL connection pool".to_string())
}

pub fn existing_fingerprint_paths() -> Result<Vec<String>, String> {
    let mut connection = pool()?
        .get()
        .map_err(|error| format!("Could not get a PostgreSQL connection: {error}"))?;

    impresario_files::table
        .inner_join(acoustid_fingerprints::table)
        .select(impresario_files::file_path)
        .load::<String>(&mut connection)
        .map_err(|error| format!("Could not load stored fingerprints: {error}"))
}

/// Persist a fingerprint; returns `false` when another scan already inserted it.
pub fn store_fingerprint(
    path: String,
    fingerprint: Vec<u8>,
    duration_seconds: i32,
) -> Result<bool, String> {
    let mut connection = pool()?
        .get()
        .map_err(|error| format!("Could not get a PostgreSQL connection: {error}"))?;

    connection
        .transaction::<bool, diesel::result::Error, _>(|connection| {
            diesel::insert_into(impresario_files::table)
                .values(impresario_files::file_path.eq(&path))
                .on_conflict(impresario_files::file_path)
                .do_nothing()
                .execute(connection)?;

            let file_id = impresario_files::table
                .filter(impresario_files::file_path.eq(&path))
                .select(impresario_files::id)
                .first::<uuid::Uuid>(connection)?;

            let inserted = diesel::insert_into(acoustid_fingerprints::table)
                .values((
                    acoustid_fingerprints::impresario_id.eq(file_id),
                    acoustid_fingerprints::fingerprint.eq(fingerprint),
                    acoustid_fingerprints::duration_seconds.eq(duration_seconds),
                ))
                .on_conflict(acoustid_fingerprints::impresario_id)
                .do_nothing()
                .execute(connection)?;

            Ok(inserted > 0)
        })
        .map_err(|error| format!("Could not store fingerprint: {error}"))
}
