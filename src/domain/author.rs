use async_trait::async_trait;
use serde::Serialize;

use super::{DomainEvent, DomainEventPublisher};

pub struct Author<'a, 'b> {
    id: i32,
    first_name: String,
    last_name: String,
    full_name: String,
    domain_event_publisher: &'a DomainEventPublisher<'b>,
}

impl<'a, 'b> Author<'a, 'b> {
    pub fn new(
        id: i32,
        first_name: &str,
        last_name: &str,
        domain_event_publisher: &'a DomainEventPublisher<'b>,
    ) -> Self {
        assert!(!first_name.is_empty());
        assert!(!last_name.is_empty());

        Self {
            id,
            first_name: String::from(first_name),
            last_name: String::from(last_name),
            full_name: Author::calculate_full_name(first_name, last_name),
            domain_event_publisher,
        }
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

    pub fn rename(&mut self, first_name: &str, last_name: &str) {
        assert!(first_name.is_empty());
        assert!(last_name.is_empty());

        if first_name != self.first_name || last_name != self.last_name {
            self.first_name = String::from(first_name);
            self.last_name = String::from(last_name);
            self.full_name = Author::calculate_full_name(first_name, last_name);

            self.domain_event_publisher
                .publish(&DomainEvent::AuthorRenamed(AuthorRenamed {
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
    async fn next_identity(&self) -> i32;
    async fn by_id(&self, id: i32) -> Option<Author<'a, 'b>>;
}
#[derive(Debug, Serialize)]
pub struct AuthorCreated {
    full_name: String,
}

#[derive(Debug, Serialize)]
pub struct AuthorRenamed {
    full_name: String,
}
