use std::{env, convert::TryFrom};

use sqlx::{SqlitePool, sqlite::SqliteRow, Row, FromRow};

mod store_error;
use store_error::StoreError;

use crate::ws::event::Event;


// impl<'r> FromRow<'r, SqliteRow> for Event {
//     fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
//         let kind = row.try_get::<String, &str>("kind")?;
//         let timestamp = row.try_get::<String, &str>("timestamp")?;
//         let device = row.try_get::<String, &str>("device")?;
//         let bundle = row.try_get::<String, &str>("bundle")?;
//     }
// }


pub struct Store(SqlitePool);

impl Store {
    pub async fn connect() -> Result<Self, StoreError> {
        let db_location = env::var("DATABASE_URL")?;
        let pool = SqlitePool::connect(&db_location).await?;
        Ok(Store(pool))
    }

    pub fn borrow_pool(&self) -> &SqlitePool {
        &self.0
    }

    pub async fn get_all_events(&self) -> Result<Vec<Event>, StoreError> {
        let mut connection = self.0.acquire().await?;

        // TODO: Implement To and From for SqliteRow -> Event
        let events: Vec<Event> = sqlx::query("SELECT * FROM Events")
            .map(|row: sqlx::sqlite::SqliteRow| {
                Event::try_from(row).unwrap()
            })
            .fetch_all(&mut connection)
            .await?;

        Ok(events)
    }
}

// pub async fn test_db_interaction() -> Result<(), Box<dyn Error>> {
//     println!("{:#?}", env::var("SOME_OLD_VAR"));
//     Ok(())
//     let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

//     // let mut conn = pool.acquire().await?;

//     // // let id = sqlx::query!(
//     // //     r#"INSERT INTO Events ( kind, timestamp, device, data ) VALUES ( "DoorOpened", "right now", "ContactSensor", "true")"#
//     // // ).execute(&mut conn)
//     // // .await?
//     // // .last_insert_rowid();

//     // let result = sqlx::query!(
//     //     r#"SELECT * FROM Events"#
//     // ).fetch_all(&mut conn)
//     // .await?;

//     // println!("{:#?}", result);

//     // Ok(())
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_db_connection() {
        let store = Store::connect().await.expect("Couldn't connect to Store");
        assert!(!store.borrow_pool().is_closed());
    }

    #[tokio::test]
    async fn test_get_all_events() {
        let store = Store::connect().await.unwrap();
        let events = store.get_all_events().await.unwrap();
        assert!(events.len() > 0);
    }
}
