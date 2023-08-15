pub mod author;
pub mod book;

use author::*;
use book::*;
use serde::Serialize;
use std::sync::RwLock;

#[derive(Debug, Serialize)]
pub enum DomainEvent {
    BookCreated(BookCreated),
    BookRenamed(BookRenamed),
    AuthorCreated(AuthorCreated),
    AuthorRenamed(AuthorRenamed),
}

impl DomainEvent {
    pub fn domain_event_name(&self) -> &'static str {
        match self {
            DomainEvent::BookCreated(_) => "book_created",
            DomainEvent::BookRenamed(_) => "book_renamed",
            DomainEvent::AuthorCreated(_) => "author_created",
            DomainEvent::AuthorRenamed(_) => "author_renamed",
        }
    }
}

pub struct DomainEventPublisher<'a> {
    handlers: RwLock<Vec<Box<dyn FnMut(&DomainEvent) + Send + Sync + 'a>>>,
}

impl<'a> DomainEventPublisher<'a> {
    pub fn new() -> Self {
        Self {
            handlers: RwLock::new(Vec::new()),
        }
    }

    pub fn publish(&self, e: &DomainEvent) {
        for handler in self.handlers.write().unwrap().iter_mut() {
            handler(e);
        }
    }

    pub fn subscribe(&self, handle: impl FnMut(&DomainEvent) + Send + Sync + 'a) {
        self.handlers.write().unwrap().push(Box::new(handle));
    }

    // pub fn subscribe_on_product_updated(
    //     &self,
    //     mut handle: impl FnMut(&BookRenamed) + Send + Sync + 'a,
    // ) {
    //     self.subscribe(move |e| {
    //         if let DomainEvent::BookRenamed(e) = e {
    //             handle(e);
    //         }
    //     })
    // }
}
