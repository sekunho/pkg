use deadpool_postgres::GenericClient;
pub use deadpool_postgres::{CreatePoolError, PoolError, Transaction};
pub use tokio_postgres::Error;

pub mod config;
pub mod handle;

pub async fn check<Client: GenericClient>(client: &Client) -> Result<(), tokio_postgres::Error> {
    let statement = client.prepare_typed("SELECT 1", &[]).await?;
    let _ = client.execute(&statement, &[]).await?;
    Ok(())
}
