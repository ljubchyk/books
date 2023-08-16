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
    use crate::{application::UoW, domain::author::Author};
    use sqlx::PgPool;

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
