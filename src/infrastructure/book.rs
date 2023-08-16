use super::DbUoW;
use crate::domain::{
    book::{Book, BookRepository},
    DomainEventPublisher,
};
use async_trait::async_trait;
use sqlx::Row;

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

        for author in book.authors() {
            let sql = format!(
                "insert into author_book(author_id, book_id) values ({}, {})",
                author,
                book.id(),
            );
            self.db.add(sql);
        }
    }

    async fn next_identity(&self) -> i32 {
        let id: (i32,) = sqlx::query_as("select nextval(pg_get_serial_sequence('book', 'id'))")
            .fetch_one(&self.db.pool)
            .await
            .unwrap();
        id.0
    }

    async fn by_id(&self, id: i32) -> Option<Book<'b, 'c>> {
        let rows = sqlx::query(
            "select * from book inner join author_book on author_book.book_id = id where id = $1",
        )
        .bind(id)
        .fetch_all(&self.db.pool)
        .await
        .unwrap();

        if rows.is_empty() {
            None
        } else {
            let authors = rows.iter().map(|r| r.get("author_id")).collect();
            let row = rows.first().unwrap();
            let book = Book::new(
                row.get("id"),
                row.get("name"),
                row.get("pages_count"),
                authors,
                self.publisher,
            );
            Some(book)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::UoW;
    use sqlx::PgPool;

    #[sqlx::test(fixtures("book"))]
    async fn create(pool: PgPool) {
        let publisher = DomainEventPublisher::new();

        let uow = DbUoW::new(pool);
        let repo = DbBookRepository::new(&uow, &publisher);

        let book = Book::new(10, "book1", 100, vec![1, 2], &publisher);
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
        assert_eq!(book.authors(), vec![1, 2]);
    }
}
