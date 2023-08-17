use super::{DomainEvent, DomainEventPublisher};
use async_trait::async_trait;
use serde::Serialize;

pub struct Book<'a, 'b> {
    id: i32,
    name: String,
    pages_count: i32,
    authors: Vec<i32>,
    publisher: &'a DomainEventPublisher<'b>,
}

impl<'a, 'b> Book<'a, 'b> {
    pub fn materialize(
        id: i32,
        name: &str,
        pages_count: i32,
        authors: Vec<i32>,
        publisher: &'a DomainEventPublisher<'b>,
    ) -> Self {
        Self {
            id,
            name: String::from(name),
            pages_count,
            authors,
            publisher,
        }
    }

    pub fn new(
        id: i32,
        name: &str,
        pages_count: i32,
        authors: Vec<i32>,
        publisher: &'a DomainEventPublisher<'b>,
    ) -> Self {
        assert!(!name.is_empty());
        assert!(pages_count.is_positive());
        assert!(!authors.is_empty());

        let book = Self {
            id,
            name: String::from(name),
            pages_count,
            authors: authors.clone(),
            publisher,
        };

        publisher.publish(&DomainEvent::BookCreated(BookCreated {
            id,
            name: String::from(name),
            pages_count,
            authors,
        }));

        book
    }

    pub fn update(&mut self, name: &str, pages_count: i32, authors: Vec<i32>) {
        assert!(!name.is_empty());
        assert!(pages_count.is_positive());
        assert!(!authors.is_empty());

        if self.name != name {
            self.name = String::from(name);
            self.publisher
                .publish(&DomainEvent::BookRenamed(BookRenamed {
                    id: self.id,
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
    fn create(&self, book: &Book);
    fn update(&self, book: &Book);
    async fn next_identity(&self) -> i32;
    async fn by_id(&self, id: i32) -> Option<Book<'a, 'b>>;
}

#[derive(Debug, Serialize)]
pub struct BookCreated {
    id: i32,
    name: String,
    pages_count: i32,
    authors: Vec<i32>,
}

#[derive(Debug, Serialize)]
pub struct BookRenamed {
    id: i32,
    name: String,
}
