use super::DbUoW;
use crate::domain::{
    book::{Book, BookRepository},
    DomainEventPublisher,
};
use async_trait::async_trait;
use sqlx::{postgres::PgRow, Row};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::UoW;
    use sqlx::PgPool;

    #[sqlx::test()]
    async fn book_create(pool: PgPool) {
        let publisher = DomainEventPublisher::new();

        let uow = DbUoW::new(pool);
        let repo = DbBookRepository::new(&uow, &publisher);

        let book = Book::new(1, "book1", 100, &publisher);
        repo.create(book);

        uow.commit().await;
    }

    #[sqlx::test(fixtures("book"))]
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
}
