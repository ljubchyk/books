use async_trait::async_trait;
use serde::Serialize;

use super::{DomainEvent, DomainEventPublisher};

pub struct Book<'a, 'b> {
    id: i32,
    name: String,
    pages_count: i32,
    domain_event_publisher: &'a DomainEventPublisher<'b>,
}

impl<'a, 'b> Book<'a, 'b> {
    pub fn new(
        id: i32,
        name: &str,
        pages_count: i32,
        domain_event_publisher: &'a DomainEventPublisher<'b>,
    ) -> Self {
        Self {
            id,
            name: String::from(name),
            pages_count,
            domain_event_publisher,
        }
    }

    pub fn rename(&mut self, name: &str) {
        if self.name != name {
            self.name = String::from(name);
            self.domain_event_publisher
                .publish(&DomainEvent::BookRenamed(BookRenamed {}));
        }
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn pages_count(&self) -> i32 {
        self.pages_count
    }
}

#[async_trait]
pub trait BookRepository<'a, 'b> {
    fn create(&self, book: Book);
    async fn next_identity(&self) -> i32;
    async fn by_id(&self, id: i32) -> Option<Book<'a, 'b>>;
}

#[derive(Debug, Serialize)]
pub struct BookCreated {
    name: String,
}

impl BookCreated {
    pub fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Serialize)]
pub struct BookRenamed {}

impl DomainEvent {
    pub fn book_created(name: &str) -> Self {
        DomainEvent::BookCreated(BookCreated::new(name))
    }
}
