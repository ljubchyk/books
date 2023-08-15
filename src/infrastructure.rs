use std::sync::RwLock;

use crate::{
    application::*,
    domain::{
        author::{Author, AuthorRepository},
        book::{Book, BookRepository},
        *,
    },
};
use async_trait::async_trait;
use sqlx::{postgres::PgRow, Executor, PgPool, Row};

pub struct DbUoW {
    pool: PgPool,
    queries: RwLock<Vec<String>>,
}

impl DbUoW {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            queries: RwLock::new(Vec::new()),
        }
    }

    pub fn add(&self, query: String) {
        self.queries.write().unwrap().push(query);
    }
}

#[async_trait]
impl UoW for DbUoW {
    async fn commit(&self) {
        let sql = self.queries.read().unwrap().join(";");
        self.pool.execute(&*sql).await.unwrap();
        self.queries.write().unwrap().clear();
    }
}

pub struct DbEventStore<'a> {
    db: &'a DbUoW,
}

impl<'a> DbEventStore<'a> {
    pub fn new(db: &'a DbUoW) -> Self {
        Self { db }
    }
}

impl<'a> EventStore for DbEventStore<'a> {
    fn append(&mut self, domain_event: &DomainEvent) {
        let name = domain_event.domain_event_name();
        let payload = serde_json::to_string(domain_event).expect("domain_event serialized");
        let stored_event = StoredEvent::new(name, &payload);
        let sql = format!(
            "insert into stored_event(name, payload) values('{}', '{}')",
            stored_event.name(),
            stored_event.playload()
        );

        self.db.add(sql);
    }
}

pub struct DbBookRepository<'a, 'b, 'c> {
    db: &'a DbUoW,
    publisher: &'b DomainEventPublisher<'c>,
}

impl<'a, 'b, 'c> DbBookRepository<'a, 'b, 'c> {
    pub fn new(db: &'a DbUoW, publisher: &'b DomainEventPublisher<'c>) -> Self {
        Self { db, publisher }
    }
}

#[async_trait]
impl<'a, 'b, 'c> BookRepository<'b, 'c> for DbBookRepository<'a, 'b, 'c> {
    fn create(&self, book: Book) {
        let sql = format!(
            "insert into book(id, name, pages_count) values ({}, '{}', {})",
            book.id(),
            book.name(),
            book.pages_count()
        );
        self.db.add(sql);
    }

    async fn next_identity(&self) -> i32 {
        let id: (i32,) = sqlx::query_as("select nextval(pg_get_serial_sequence('book', 'id'))")
            .fetch_one(&self.db.pool)
            .await
            .unwrap();
        id.0
    }

    async fn by_id(&self, id: i32) -> Option<Book<'b, 'c>> {
        let row = sqlx::query("select * from book where id = $1")
            .bind(id)
            .fetch_optional(&self.db.pool)
            .await
            .unwrap();
        row.map(|row: PgRow| {
            Book::new(
                row.get("id"),
                row.get("name"),
                row.get("pages_count"),
                self.publisher,
            )
        })
    }
}

pub struct DbAuthorRepository<'a, 'b, 'c> {
    db: &'a DbUoW,
    publisher: &'b DomainEventPublisher<'c>,
}

impl<'a, 'b, 'c> DbAuthorRepository<'a, 'b, 'c> {
    pub fn new(db: &'a DbUoW, publisher: &'b DomainEventPublisher<'c>) -> Self {
        Self { db, publisher }
    }
}

#[async_trait]
impl<'a, 'b, 'c> AuthorRepository<'b, 'c> for DbAuthorRepository<'a, 'b, 'c> {
    fn create(&self, author: &Author) {
        let sql = format!(
            "insert into author(id, first_name, last_name, full_name) values ({}, '{}', '{}', '{}')",
            author.id(),
            author.first_name(),
            author.last_name(),
            author.full_name(),
        );
        self.db.add(sql);
    }

    async fn next_identity(&self) -> i32 {
        let id: (i32,) = sqlx::query_as("select nextval(pg_get_serial_sequence('author', 'id'))")
            .fetch_one(&self.db.pool)
            .await
            .unwrap();
        id.0
    }

    async fn by_id(&self, id: i32) -> Option<Author<'b, 'c>> {
        let author = sqlx::query("select * from author where id = $1")
            .bind(id)
            .map(|row: PgRow| {
                Author::new(
                    row.get("id"),
                    row.get("first_name"),
                    row.get("last_name"),
                    self.publisher,
                )
            })
            .fetch_optional(&self.db.pool)
            .await
            .unwrap();
        author
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test()]
    async fn book_create(pool: PgPool) {
        let publisher = DomainEventPublisher::new();

        let uow = DbUoW::new(pool);
        let repo = DbBookRepository::new(&uow, &publisher);

        let book = Book::new(1, "book1", 100, &publisher);
        repo.create(book);

        uow.commit().await;
    }

    #[sqlx::test(fixtures("books"))]
    async fn get(pool: PgPool) {
        let publisher = DomainEventPublisher::new();
        let uow = DbUoW::new(pool);
        let repo = DbBookRepository::new(&uow, &publisher);

        let book = repo.by_id(1).await;
        assert!(book.is_some());

        let book = book.unwrap();
        assert_eq!(book.id(), 1);
        assert_eq!(book.name(), "book1");
        assert_eq!(book.pages_count(), 100);
    }

    #[sqlx::test]
    fn stored_event_test(pool: PgPool) {
        let uow = DbUoW::new(pool);
        let mut event_store = DbEventStore::new(&uow);

        event_store.append(&DomainEvent::book_created("book1"));

        uow.commit().await;
    }

    #[sqlx::test]
    fn author_test(pool: PgPool) {
        let uow = DbUoW::new(pool);
        let publisher = DomainEventPublisher::new();
        let repo = DbAuthorRepository::new(&uow, &publisher);

        let author = Author::new(1, "fn", "ln", &publisher);
        repo.create(&author);

        uow.commit().await;

        let author_db = repo.by_id(1).await;
        assert!(author_db.is_some());

        let author_db = author_db.unwrap();
        assert_eq!(author_db.first_name(), author.first_name());
        assert_eq!(author_db.last_name(), author.last_name());
        assert_eq!(author_db.full_name(), author.full_name());
    }
}
