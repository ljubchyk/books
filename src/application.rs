pub mod author;
pub mod book;

use crate::domain::{DomainEvent, DomainEventPublisher};
use async_trait::async_trait;

fn begin<'a>(publisher: &DomainEventPublisher<'a>, event_store: &'a mut dyn EventStore) {
    publisher.subscribe(|e| event_store.append(e));
}

async fn success(uow: &mut impl UoW) {
    uow.commit().await
}

#[derive(Debug, Clone)]
pub struct StoredEvent {
    name: String,
    payload: String,
}

impl StoredEvent {
    pub fn new(name: &str, payload: &str) -> Self {
        Self {
            name: String::from(name),
            payload: String::from(payload),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn playload(&self) -> &str {
        &self.payload
    }
}

#[async_trait]
pub trait UoW {
    async fn commit(&self);
}

pub trait EventStore: Send + Sync {
    fn append(&mut self, domain_event: &DomainEvent);
}
