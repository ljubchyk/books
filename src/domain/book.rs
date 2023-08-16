use super::{DomainEvent, DomainEventPublisher};
use async_trait::async_trait;
use serde::Serialize;

pub struct Book<'a, 'b> {
    id: i32,
    name: String,
    pages_count: i32,
    authors: Vec<i32>,
    domain_event_publisher: &'a DomainEventPublisher<'b>,
}

impl<'a, 'b> Book<'a, 'b> {
    pub fn new(
        id: i32,
        name: &str,
        pages_count: i32,
        authors: Vec<i32>,
        domain_event_publisher: &'a DomainEventPublisher<'b>,
    ) -> Self {
        assert!(!name.is_empty());
        assert!(pages_count.is_positive());
        assert!(!authors.is_empty());

        Self {
            id,
            name: String::from(name),
            pages_count,
            authors,
            domain_event_publisher,
        }
    }

    pub fn update(&mut self, name: &str, pages_count: i32, authors: Vec<i32>) {
        assert!(!name.is_empty());
        assert!(pages_count.is_positive());
        assert!(!authors.is_empty());

        if self.name != name {
            self.name = String::from(name);
            self.domain_event_publisher
                .publish(&DomainEvent::BookCreated(BookCreated {
                    name: String::from(name),
                }));
        }

        self.pages_count = pages_count;
        self.authors = authors;
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

    pub fn authors(&self) -> &[i32] {
        &self.authors
    }
}

#[async_trait]
pub trait BookRepository<'a, 'b> {
    fn create(&self, book: Book);
    fn update(&self, book: Book);
    async fn next_identity(&self) -> i32;
    async fn by_id(&self, id: i32) -> Option<Book<'a, 'b>>;
}

#[derive(Debug, Serialize)]
pub struct BookCreated {
    name: String,
}

#[derive(Debug, Serialize)]
pub struct BookRenamed {}
