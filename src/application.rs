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

pub mod book_application {
    use super::*;
    use crate::domain::book::{Book, BookRepository};

    pub async fn create<'a>(
        name: &str,
        pages_count: i32,
        book_repository: &mut impl BookRepository<'_, '_>,
        event_store: &'a mut impl EventStore,
        uow: &mut impl UoW,
    ) {
        let publisher = DomainEventPublisher::new();
        begin(&publisher, event_store);

        let id = book_repository.next_identity().await;
        let book = Book::new(id, name, pages_count, &publisher);
        book_repository.create(book);

        success(uow).await;
    }
}

pub mod athor_application {
    use super::*;
    use crate::domain::author::{Author, AuthorRepository};

    pub async fn create(
        first_name: &str,
        last_name: &str,
        author_repository: &mut impl AuthorRepository<'_, '_>,
        event_store: &mut impl EventStore,
        uow: &mut impl UoW,
    ) {
        let publisher = DomainEventPublisher::new();
        begin(&publisher, event_store);

        let id = author_repository.next_identity().await;
        let author = Author::new(id, first_name, last_name, &publisher);
        author_repository.create(&author);

        success(uow).await;
    }
}
