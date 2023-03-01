use std::env;
use log::*;

use sqlx::SqlitePool;

use crate::model::Event;

#[derive(thiserror::Error, Debug)]
pub enum StoreError {
    #[error("The DATABASE_URL environment variable is not set, or the location is incorrect")]
    BadDBLocation(#[from] std::env::VarError),
    /// A wrapper around any SQLx Error
    #[error("SQLx error: {0}")]
    SQLxError(#[from] sqlx::Error),
    /// A general database decode error that can turn into a SQLx Error::Decode error
    #[error("Could not decode value from database: {0}")]
    DecodeError(String)
}

impl StoreError {
    pub fn into_sqlx_decode_error(self) -> sqlx::Error {
        sqlx::Error::Decode(Box::new(self))
    }
}


pub struct Store(SqlitePool);

impl Store {
    /// Connects to a Sqlite database. You must set the `DATABASE_URL` environment variable.
    /// 
    /// Example:
    /// ```
    /// $ export DATABASE_URL=sqlite:my_db_file.db
    /// ```
    pub async fn connect() -> Result<Self, StoreError> {
        let db_location;
        #[cfg(debug_assertions)] {
            db_location = env::var("DATABASE_URL")?;
        }
        #[cfg(not(debug_assertions))] {
            db_location = String::from("sqlite:modkit.db");
        }
        info!("Using {db_location} as database location");
        let pool = SqlitePool::connect(&db_location).await?;
        Ok(Store(pool))
    }

    /// Borrows the connection pool
    pub fn borrow_pool(&self) -> &SqlitePool {
        &self.0
    }

    /// Gets all events from the DB and returns them as a vector
    pub async fn get_all_events(&self) -> Result<Vec<Event>, StoreError> {
        let mut connection = self.0.acquire().await?;
        let events: Vec<Event> = sqlx::query_as::<_, Event>("SELECT * FROM Events;")
            .fetch_all(&mut connection)
            .await?;
        Ok(events)
    }

    /// Write a single event to the db
    pub async fn write_event(&self, event: Event) -> Result<(), StoreError> {
        let mut connection = self.0.acquire().await?;

        // Serialize the event details into strings
        // Event Kind
        let event_kind = format!("{}", event.kind());
        // Event timestamp
        let timestamp = event.timestamp();
        // Event device (if any)
        let device = match event.device_type() {
            Some(d) => format!("{d}"),
            None => format!("None"),
        };
        // Event data (if any)
        let data = match event.data() {
            Some(d) => {
                let unescaped = d.to_json().unwrap();
                unescaped.replace(r#"""#, r#""""#)
            }
            None => format!("None"),
        };

        // Insert into table
        sqlx::query!(
            "INSERT INTO Events (kind, timestamp, device, data) VALUES (?, ?, ?, ?);",
            event_kind,
            timestamp,
            device,
            data
        )
        .execute(&mut connection)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{drivers::device::DeviceType, model::EventKind};

    use super::*;

    async fn clear_db() {
        let store = Store::connect().await.unwrap();
        sqlx::query("DELETE FROM Events;")
            .execute(store.borrow_pool())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_db_connection() {
        clear_db().await;

        let store = Store::connect().await.expect("Couldn't connect to Store");
        assert!(!store.borrow_pool().is_closed());
    }

    #[tokio::test]
    async fn test_get_all_events() {
        clear_db().await;

        let store = Store::connect().await.unwrap();
        sqlx::query(r#"INSERT INTO Events (kind, timestamp, device, data)
            VALUES ("DoorOpened", "2023-02-26 00:23:23.881440460 -06:00", "ContactSensor", "{""ContactSensor"":{""open"":true}}");"#)
            .execute(store.borrow_pool()).await.unwrap();

        let events = store.get_all_events().await.unwrap();
        println!("{:#?}", events);
        assert!(events.len() > 0);
        assert!(events.get(0).is_some());
        let e = events.get(0).unwrap();
        assert_eq!(e.kind(), &EventKind::DoorOpened);
        assert!(e.timestamp().contains("2023-02-26"));
        assert!(e.data().is_some());
        assert_eq!(e.device_type().unwrap(), &DeviceType::ContactSensor);
    }

    #[tokio::test]
    async fn test_write_event() {
        let store = Store::connect().await.unwrap();

        let event = Event::new(EventKind::MailDelivered, None, None);
        store.write_event(event).await.unwrap();
    }
}
