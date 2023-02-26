use std::{convert::TryFrom, env};

use sqlx::{sqlite::SqliteRow, Sqlite, SqlitePool};

mod store_error;
pub use store_error::StoreError;

use crate::model::Event;

// use crate::ws::event::Event;

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
        let events: Vec<Event> = sqlx::query_as::<_, Event>("SELECT * FROM Events;")
            .fetch_all(&mut connection)
            .await?;
        Ok(events)
    }

    pub async fn write_event(&self, event: Event) -> Result<(), StoreError> {
        let mut connection = self.0.acquire().await?;
        let event_kind = format!("{}", event.kind());
        let timestamp = event.timestamp();
        let device = match event.device_type() {
            Some(d) => format!("{d}"),
            None => format!("None"),
        };
        let data = match event.data() {
            Some(d) => {
                let unescaped = d.to_json().unwrap();
                unescaped.replace(r#"""#, r#""""#)
            }
            None => format!("None"),
        };
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
