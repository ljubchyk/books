use super::{DomainEvent, DomainEventPublisher};
use async_trait::async_trait;
use serde::Serialize;

pub struct Author<'a, 'b> {
    id: i32,
    first_name: String,
    last_name: String,
    full_name: String,
    publisher: &'a DomainEventPublisher<'b>,
}

impl<'a, 'b> Author<'a, 'b> {
    pub fn materialize(
        id: i32,
        first_name: &str,
        last_name: &str,
        full_name: &str,
        publisher: &'a DomainEventPublisher<'b>,
    ) -> Self {
        Self {
            id,
            first_name: String::from(first_name),
            last_name: String::from(last_name),
            full_name: String::from(full_name),
            publisher,
        }
    }

    pub fn new(
        id: i32,
        first_name: &str,
        last_name: &str,
        publisher: &'a DomainEventPublisher<'b>,
    ) -> Self {
        assert!(!first_name.is_empty());
        assert!(!last_name.is_empty());

        let author = Self {
            id,
            first_name: String::from(first_name),
            last_name: String::from(last_name),
            full_name: Author::calculate_full_name(first_name, last_name),
            publisher,
        };

        publisher.publish(&DomainEvent::AuthorCreated(AuthorCreated {
            id,
            first_name: String::from(first_name),
            last_name: String::from(last_name),
            full_name: String::from(&author.full_name),
        }));

        author
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn first_name(&self) -> &str {
        &self.first_name
    }

    pub fn last_name(&self) -> &str {
        &self.last_name
    }

    pub fn full_name(&self) -> &str {
        &self.full_name
    }

    pub fn update(&mut self, first_name: &str, last_name: &str) {
        assert!(!first_name.is_empty());
        assert!(!last_name.is_empty());

        if first_name != self.first_name || last_name != self.last_name {
            self.first_name = String::from(first_name);
            self.last_name = String::from(last_name);
            self.full_name = Author::calculate_full_name(first_name, last_name);

            self.publisher
                .publish(&DomainEvent::AuthorCreated(AuthorCreated {
                    id: self.id,
                    first_name: String::from(first_name),
                    last_name: String::from(last_name),
                    full_name: String::from(&self.full_name),
                }));
        }
    }

    fn calculate_full_name(first_name: &str, last_name: &str) -> String {
        format!("{} {}", first_name, last_name)
    }
}

#[async_trait]
pub trait AuthorRepository<'a, 'b> {
    fn create(&self, author: &Author);
    fn update(&self, author: &Author);
    async fn next_identity(&self) -> i32;
    async fn by_id(&self, id: i32) -> Option<Author<'a, 'b>>;
}

#[derive(Debug, Serialize)]
pub struct AuthorCreated {
    id: i32,
    first_name: String,
    last_name: String,
    full_name: String,
}

#[derive(Debug, Serialize)]
pub struct AuthorRenamed {
    id: i32,
    first_name: String,
    last_name: String,
    full_name: String,
}
