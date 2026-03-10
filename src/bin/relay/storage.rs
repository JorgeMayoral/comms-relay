use std::time::Duration;

use anyhow::{Context, Result};
use comms::publication::Publication;
use jiff::{Timestamp, Zoned, tz::TimeZone};
use sqlx::{
    Decode, Encode, PgPool, Postgres,
    encode::IsNull,
    error::BoxDynError,
    postgres::{PgArgumentBuffer, PgPoolOptions, PgTypeInfo, PgValueRef},
};
use ulid::Ulid;

pub struct PgStorage(PgPool);

impl PgStorage {
    pub async fn create(url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(5))
            .connect(url)
            .await
            .context("connect to postgres")?;
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .context("run migrations")?;
        Ok(Self(pool))
    }

    #[tracing::instrument(skip(self), err)]
    pub async fn insert_publication(&self, publication: &Publication) -> Result<()> {
        sqlx::query!(
            r#"INSERT INTO publications
                (id, content, pub_date, mastodon_id, mastodon_url, bluesky_id, bluesky_url)
                VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
            publication.id().to_string(),
            publication.content(),
            PgZoned(publication.pub_date().clone()) as PgZoned,
            publication.mastodon_id(),
            publication.mastodon_url(),
            publication.bluesky_id(),
            publication.bluesky_url(),
        )
        .execute(&self.0)
        .await
        .context("insert publication")?;
        tracing::debug!("publication inserted");
        Ok(())
    }

    #[tracing::instrument(skip(self), err)]
    pub async fn get_publication(&self, id: Ulid) -> Result<Option<Publication>> {
        let row = sqlx::query!(
            r#"SELECT id, content, pub_date as "pub_date: PgZoned",
               mastodon_id, mastodon_url, bluesky_id, bluesky_url
               FROM publications WHERE id = $1"#,
            id.to_string(),
        )
        .fetch_optional(&self.0)
        .await
        .context("get publication")?;
        tracing::debug!(found = row.is_some(), "publication fetched");
        Ok(row.map(|r| {
            Publication::new(
                r.id.parse().expect("ULID in DB was written by us"),
                r.content,
                r.pub_date.0,
                r.mastodon_id,
                r.mastodon_url,
                r.bluesky_id,
                r.bluesky_url,
            )
        }))
    }

    #[tracing::instrument(skip(self), err)]
    pub async fn list_publications(&self, results: i64, offset: i64) -> Result<Vec<Publication>> {
        let rows = sqlx::query!(
            r#"SELECT id, content, pub_date as "pub_date: PgZoned",
               mastodon_id, mastodon_url, bluesky_id, bluesky_url
               FROM publications ORDER BY pub_date DESC
               LIMIT $1 OFFSET $2"#,
            results,
            offset,
        )
        .fetch_all(&self.0)
        .await
        .context("list publications")?;

        tracing::debug!(count = rows.len(), "publications listed");
        Ok(rows
            .into_iter()
            .map(|r| {
                Publication::new(
                    r.id.parse().expect("ULID in DB was written by us"),
                    r.content,
                    r.pub_date.0,
                    r.mastodon_id,
                    r.mastodon_url,
                    r.bluesky_id,
                    r.bluesky_url,
                )
            })
            .collect())
    }

    #[tracing::instrument(skip(self), err)]
    pub async fn count_publications(&self) -> Result<i64> {
        let row = sqlx::query!("SELECT COUNT(*) as count FROM publications")
            .fetch_one(&self.0)
            .await
            .context("count publications")?;
        Ok(row.count.unwrap_or(0))
    }

    #[tracing::instrument(skip(self), err)]
    pub async fn delete_publication(&self, id: Ulid) -> Result<bool> {
        let row = sqlx::query!(r#"DELETE FROM publications WHERE id = $1"#, id.to_string(),)
            .execute(&self.0)
            .await
            .context("delete publication")?;
        let deleted = row.rows_affected() > 0;
        tracing::debug!(deleted, "publication delete attempted");
        Ok(deleted)
    }
}

const PG_EPOCH_OFFSET_MICROS: i64 = 946_684_800 * 1_000_000;

#[derive(Debug)]
struct PgZoned(Zoned);

impl sqlx::Type<Postgres> for PgZoned {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("TIMESTAMPTZ")
    }
}

impl Encode<'_, Postgres> for PgZoned {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        let pg_micros = self.0.timestamp().as_microsecond() - PG_EPOCH_OFFSET_MICROS;
        <i64 as Encode<'_, Postgres>>::encode_by_ref(&pg_micros, buf)
    }
}

impl<'r> Decode<'r, Postgres> for PgZoned {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let pg_micros = <i64 as Decode<'_, Postgres>>::decode(value)?;
        let zoned = Timestamp::from_microsecond(pg_micros + PG_EPOCH_OFFSET_MICROS)?
            .to_zoned(TimeZone::UTC);
        Ok(PgZoned(zoned))
    }
}
