use super::DbUoW;
use crate::domain::{
    author::{Author, AuthorRepository},
    DomainEventPublisher,
};
use async_trait::async_trait;
use sqlx::{postgres::PgRow, Row};

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

    fn update(&self, author: &Author) {
        let sql = format!(
            "update author set first_name = '{}', last_name = '{}', full_name = '{}' where id = {}",
            author.first_name(),
            author.last_name(),
            author.full_name(),
            author.id(),
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
mod test {
    use super::*;
    use crate::{application::UoW, domain::author::Author};
    use sqlx::PgPool;

    #[sqlx::test(fixtures("author"))]
    fn create(pool: PgPool) {
        let uow = DbUoW::new(pool);
        let publisher = DomainEventPublisher::new();
        let repo = DbAuthorRepository::new(&uow, &publisher);

        let author_id = 10;
        let author = Author::new(author_id, "f", "l", &publisher);
        repo.create(&author);

        uow.commit().await;

        let rows = sqlx::query("select * from author where id = $1")
            .bind(author_id)
            .fetch_all(&uow.pool)
            .await
            .unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get::<i32, _>("id"), author_id);
        assert_eq!(rows[0].get::<&str, _>("first_name"), "f");
        assert_eq!(rows[0].get::<&str, _>("last_name"), "l");
        assert_eq!(rows[0].get::<&str, _>("full_name"), "f l");
    }

    #[sqlx::test(fixtures("author"))]
    fn update(pool: PgPool) {
        let uow = DbUoW::new(pool);
        let publisher = DomainEventPublisher::new();
        let repo = DbAuthorRepository::new(&uow, &publisher);

        let author_id = 1;
        let author = Author::new(author_id, "f1-renamed", "l1-renamed", &publisher);
        repo.update(&author);

        uow.commit().await;

        let rows = sqlx::query("select * from author where id = $1")
            .bind(author_id)
            .fetch_all(&uow.pool)
            .await
            .unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get::<i32, _>("id"), author_id);
        assert_eq!(rows[0].get::<&str, _>("first_name"), "f1-renamed");
        assert_eq!(rows[0].get::<&str, _>("last_name"), "l1-renamed");
        assert_eq!(rows[0].get::<&str, _>("full_name"), "f1-renamed l1-renamed");
    }

    #[sqlx::test(fixtures("author"))]
    async fn get(pool: PgPool) {
        let publisher = DomainEventPublisher::new();
        let uow = DbUoW::new(pool);
        let repo = DbAuthorRepository::new(&uow, &publisher);

        let author_id = 1;
        let author = repo.by_id(author_id).await;
        assert!(author.is_some());

        let author = author.unwrap();
        assert_eq!(author.id(), author_id);
        assert_eq!(author.first_name(), "f1");
        assert_eq!(author.last_name(), "l1");
        assert_eq!(author.full_name(), "f1 l1");
    }
}
