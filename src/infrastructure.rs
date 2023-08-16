pub mod author;
pub mod book;

use async_trait::async_trait;
use sqlx::{Executor, PgPool};
use std::sync::RwLock;

use crate::{
    application::{EventStore, StoredEvent, UoW},
    domain::DomainEvent,
};

pub struct DbUoW {
    pool: PgPool,
    queries: RwLock<Vec<String>>,
}

impl DbUoW {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            queries: RwLock::new(Vec::new()),
        }
    }

    pub fn add(&self, query: String) {
        self.queries.write().unwrap().push(query);
    }
}

#[async_trait]
impl UoW for DbUoW {
    async fn commit(&self) {
        let sql = self.queries.read().unwrap().join(";");
        self.pool.execute(&*sql).await.unwrap();
        self.queries.write().unwrap().clear();
    }
}

pub struct DbEventStore<'a> {
    db: &'a DbUoW,
}

impl<'a> DbEventStore<'a> {
    pub fn new(db: &'a DbUoW) -> Self {
        Self { db }
    }
}

impl<'a> EventStore for DbEventStore<'a> {
    fn append(&mut self, domain_event: &DomainEvent) {
        let name = domain_event.domain_event_name();
        let payload = serde_json::to_string(domain_event).expect("domain_event serialized");
        let stored_event = StoredEvent::new(name, &payload);
        let sql = format!(
            "insert into stored_event(name, payload) values('{}', '{}')",
            stored_event.name(),
            stored_event.playload()
        );

        self.db.add(sql);
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::domain::book::BookCreated;

//     use super::*;

//     #[sqlx::test]
//     fn stored_event_test(pool: PgPool) {
//         let uow = DbUoW::new(pool);
//         let mut event_store = DbEventStore::new(&uow);

//         event_store.append(&DomainEvent::BookCreated(BookCreated {
//             name: String::from("book1"),
//         }));

//         uow.commit().await;
//     }
// }
