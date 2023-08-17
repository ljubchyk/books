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
    fn create(&self, book: &Book) {
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
                book.id()
            );
            self.db.add(sql);
        }
    }

    fn update(&self, book: &Book) {
        let sql = format!(
            "update book set name = '{}', pages_count = {} where id = {}",
            book.name(),
            book.pages_count(),
            book.id()
        );
        self.db.add(sql);

        let sql = format!("delete from author_book where book_id = {}", book.id());
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
        // let id: (i64,) = sqlx::query_as("select nextval(pg_get_serial_sequence('book', 'id'))")
        let id: (i64,) = sqlx::query_as("select nextval('book_id_seq')")
            .fetch_one(&self.db.pool)
            .await
            .unwrap();
        id.0 as i32
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
            rows.first().map(|row| {
                Book::materialize(
                    row.get("id"),
                    row.get("name"),
                    row.get("pages_count"),
                    authors,
                    self.publisher,
                )
            })
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::application::UoW;
    use sqlx::PgPool;

    #[sqlx::test(fixtures("book"))]
    fn next_identity(pool: PgPool) {
        sqlx::query("select setval('book_id_seq', 1)")
            .execute(&pool)
            .await
            .unwrap();

        let uow = DbUoW::new(pool);
        let publisher = DomainEventPublisher::new();
        let repo = DbBookRepository::new(&uow, &publisher);
        let id = repo.next_identity().await;

        assert_eq!(id, 2);
    }

    #[sqlx::test(fixtures("book"))]
    async fn create(pool: PgPool) {
        let publisher = DomainEventPublisher::new();

        let uow = DbUoW::new(pool);
        let repo = DbBookRepository::new(&uow, &publisher);

        let book_id = 10;
        let book = Book::materialize(book_id, "book10", 100, vec![1, 2], &publisher);
        repo.create(&book);

        uow.commit().await;

        let rows = sqlx::query(
            "select * from book inner join author_book on author_book.book_id = id where id = $1",
        )
        .bind(book_id)
        .fetch_all(&uow.pool)
        .await
        .unwrap();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].get::<i32, _>("id"), 10);
        assert_eq!(rows[0].get::<&str, _>("name"), "book10");
        assert_eq!(rows[0].get::<i32, _>("pages_count"), 100);
        assert_eq!(rows[0].get::<i32, _>("author_id"), 1);
        assert_eq!(rows[1].get::<i32, _>("author_id"), 2);
    }

    #[sqlx::test(fixtures("book"))]
    async fn update(pool: PgPool) {
        let publisher = DomainEventPublisher::new();

        let uow = DbUoW::new(pool);
        let repo = DbBookRepository::new(&uow, &publisher);

        let book_id = 1;
        let book = Book::materialize(book_id, "book1-renamed", 10, vec![1], &publisher);
        repo.update(&book);

        uow.commit().await;

        let rows = sqlx::query(
            "select * from book inner join author_book on author_book.book_id = id where id = $1",
        )
        .bind(book_id)
        .fetch_all(&uow.pool)
        .await
        .unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get::<i32, _>("id"), 1);
        assert_eq!(rows[0].get::<&str, _>("name"), "book1-renamed");
        assert_eq!(rows[0].get::<i32, _>("pages_count"), 10);
        assert_eq!(rows[0].get::<i32, _>("author_id"), 1);
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
