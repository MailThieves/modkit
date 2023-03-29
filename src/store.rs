use log::*;

use sqlx::SqlitePool;

use crate::model::{Event, EventKind};

pub const DB_LOCATION: &'static str = "sqlite:modkit.db";

#[derive(thiserror::Error, Debug)]
pub enum StoreError {
    #[error("File database file {DB_LOCATION} does not exist or cannot be found")]
    BadDBLocation(#[from] std::env::VarError),
    /// A wrapper around any SQLx Error
    #[error("SQLx error: {0}")]
    SQLxError(#[from] sqlx::Error),
    /// A general database decode error that can turn into a SQLx Error::Decode error
    #[error("Could not decode value from database: {0}")]
    DecodeError(String),
    #[error(
        "A mail status event (MailDelivered/MailPickedUp) could not be found in the database: {0}"
    )]
    MailStatusNotFound(sqlx::Error),
}

impl StoreError {
    pub fn into_sqlx_decode_error(self) -> sqlx::Error {
        sqlx::Error::Decode(Box::new(self))
    }
}

pub struct Store(SqlitePool);

impl Store {
    /// Connects to a Sqlite database.
    pub async fn connect() -> Result<Self, StoreError> {
        trace!("Using {DB_LOCATION} as database location");
        let pool = SqlitePool::connect(DB_LOCATION).await?;
        Ok(Store(pool))
    }

    /// Borrows the connection pool
    #[allow(unused)]
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

    #[allow(unused)]
    pub async fn nuke(&self) -> Result<(), StoreError> {
        let mut connection = self.0.acquire().await?;
        sqlx::query("DELETE FROM Events;")
            .execute(&mut connection)
            .await?;
        Ok(())
    }

    pub async fn get_mail_status(&self) -> Result<Event, StoreError> {
        let mut connection = self.0.acquire().await?;

        let latest = sqlx::query_as::<_, Event>(
            r#"
            SELECT * FROM Events
            WHERE ID = (SELECT MAX(ID) FROM Events
                WHERE kind = 'MailDelivered'
                OR kind = 'MailPickedUp'
                ORDER BY timestamp
            );"#,
        )
        .fetch_one(&mut connection)
        .await
        .map_err(|e| StoreError::MailStatusNotFound(e));

        latest
    }

    /// Write a single event to the db
    pub async fn write_event(&self, event: Event) -> Result<(), StoreError> {
        // We want to silently skip writing the EventHistory event because all it does is return
        // a list of previous events. We could get a nasty loop of recursive event-writing
        // to the db. Bad news.
        if event.kind() == &EventKind::EventHistory {
            return Ok(());
        }

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

    #[tokio::test]
    async fn test_db_connection() {
        let store = Store::connect().await.expect("Couldn't connect to Store");
        assert!(!store.borrow_pool().is_closed());
    }

    #[tokio::test]
    async fn test_get_all_events() {
        let store = Store::connect().await.unwrap();
        
        store.nuke().await.unwrap();

        sqlx::query(r#"INSERT INTO Events (kind, timestamp, device, data)
            VALUES ("DoorOpened", 123456, "ContactSensor", "{""ContactSensor"":{""open"":true}}");"#)
            .execute(store.borrow_pool()).await.unwrap();

        let events = store.get_all_events().await.unwrap();
        println!("{:#?}", events);
        assert!(events.len() > 0);
        assert!(events.get(0).is_some());
        let e = events.get(0).unwrap();
        assert_eq!(e.kind(), &EventKind::DoorOpened);
        assert!(e.data().is_some());
        assert_eq!(e.device_type().unwrap(), &DeviceType::ContactSensor);

        store.nuke().await.unwrap();
    }

    #[tokio::test]
    async fn test_write_event() {
        let store = Store::connect().await.unwrap();
        
        store.nuke().await.unwrap();
        
        let event = Event::new(EventKind::MailDelivered, None, None);
        store.write_event(event).await.unwrap();

        store.nuke().await.unwrap();
    }

    #[tokio::test]
    async fn test_get_latest_mail_status() {
        let store = Store::connect().await.unwrap();
        
        store.nuke().await.unwrap();

        let events = vec![
            Event::new(EventKind::MailDelivered, None, None),
            Event::new(EventKind::MailPickedUp, None, None),
            Event::new(EventKind::MailDelivered, None, None),
            Event::new(EventKind::MailPickedUp, None, None),
        ];

        for e in events {
            store.write_event(e).await.unwrap();
        }

        let latest = store.get_mail_status().await;
        assert!(latest.is_ok());
        assert_eq!(latest.unwrap().kind(), &EventKind::MailPickedUp);

        store.nuke().await.unwrap();

        assert!(store.get_mail_status().await.is_err());
    }
}
