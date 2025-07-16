use async_trait::async_trait;
use sqlx::postgres::PgPool;

#[async_trait]
pub trait Persistable {
    async fn save_to_db(&self, pool: &PgPool) -> Result<(), sqlx::Error>;
}
