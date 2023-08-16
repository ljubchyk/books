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
