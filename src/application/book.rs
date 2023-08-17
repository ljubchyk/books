use super::*;
use crate::domain::book::{Book, BookRepository};

pub async fn create<'a>(
    name: &str,
    pages_count: i32,
    authors: Vec<i32>,
    book_repository: &mut impl BookRepository<'_, '_>,
    event_store: &'a mut impl EventStore,
    uow: &mut impl UoW,
) {
    let publisher = DomainEventPublisher::new();
    begin(&publisher, event_store);

    let id = book_repository.next_identity().await;
    let book = Book::new(id, name, pages_count, authors, &publisher);
    book_repository.create(&book);

    success(uow).await;
}
