use crate::domain::{
    book::{BookCreated, BookRenamed},
    DomainEventPublisher,
};

pub fn create(publisher: &DomainEventPublisher) {
    publisher.subscribe(|e| match e {
        crate::domain::DomainEvent::BookCreated(e) => on_book_created(e),
        crate::domain::DomainEvent::BookRenamed(e) => on_book_renamed(e),
        _ => {}
    })
}

fn on_book_created(e: &BookCreated) {
    println!("{:?}", e);
}

fn on_book_renamed(e: &BookRenamed) {
    println!("{:?}", e);
}
