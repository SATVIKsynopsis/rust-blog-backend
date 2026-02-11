use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::models::{Comment, Like, Post, User};

#[derive(Debug, Clone)]
pub struct DBClient {
    pub pool: Pool<Postgres>,
}

impl DBClient {
    pub fn new(pool: Pool<Postgres>) -> Self {
        DBClient { pool }
    }
}

#[async_trait]
pub trait UserExt {
    async fn get_user(
        &self,
        id: Option<Uuid>,
        name: Option<&str>,
        email: Option<&str>,
    ) -> Result<Option<User>, sqlx::Error>;

    async fn get_users(&self, page: u32, limit: u32) -> Result<Vec<User>, sqlx::Error>;

    async fn save_user<T: Into<String> + Send>(
        &self,
        username: T,
        name: T,
        email: T,
        password: T,
    ) -> Result<User, sqlx::Error>;

    async fn update_user_name<T: Into<String> + Send>(
        &self,
        user_id: Uuid,
        name: T,
    ) -> Result<User, sqlx::Error>;

    async fn update_user_password(
        &self,
        user_id: Uuid,
        new_password: String,
    ) -> Result<User, sqlx::Error>;

    async fn create_post<T: Into<String> + Send>(
        &self,
        author_id: Uuid,
        title: T,
        content: T,
    ) -> Result<Post, sqlx::Error>;

    async fn like_post(&self, user_id: Uuid, post_id: Uuid) -> Result<Like, sqlx::Error>;

    async fn create_comment<T: Into<String> + Send>(
        &self,
        post_id: Uuid,
        user_id: Uuid,
        content: T,
    ) -> Result<Comment, sqlx::Error>;

    async fn get_post(&self, post_id: Uuid) -> Result<Option<Post>, sqlx::Error>;

    async fn get_posts(&self, page: u32, limit: usize) -> Result<Vec<Post>, sqlx::Error>;

    async fn update_post(
        &self,
        post_id: Uuid,
        user_id: Uuid,
        title: &str,
        content: &str,
    ) -> Result<Post, sqlx::Error>;

    async fn delete_post(&self, post_id: Uuid, user_id: Uuid) -> Result<(), sqlx::Error>;

    async fn get_user_posts(&self, author_id: Uuid) -> Result<Vec<Post>, sqlx::Error>;

    async fn increment_view(&self, post_id: Uuid) -> Result<(), sqlx::Error>;
}

#[async_trait]
impl UserExt for DBClient {
    async fn get_user(
        &self,
        user_id: Option<Uuid>,
        name: Option<&str>,
        email: Option<&str>,
    ) -> Result<Option<User>, sqlx::Error> {
        let mut user: Option<User> = None;

        if let Some(user_id) = user_id {
            user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1 LIMIT 1", user_id)
                .fetch_optional(&self.pool)
                .await?
        } else if let Some(name) = name {
            user = sqlx::query_as!(User, "SELECT * FROM users WHERE name = $1 LIMIT 1", name)
                .fetch_optional(&self.pool)
                .await?
        } else if let Some(email) = email {
            user = sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1 LIMIT 1", email)
                .fetch_optional(&self.pool)
                .await?
        }

        Ok(user)
    }

    async fn save_user<T: Into<String> + Send>(
        &self,
        username: T,
        name: T,
        email: T,
        password: T,
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            "INSERT INTO users (username, name, email, password)
             VALUES ($1, $2, $3, $4)
             RETURNING id, name, username, email, bio, password, created_at, updated_at",
            username.into(),
            name.into(),
            email.into(),
            password.into()
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    async fn update_user_name<T: Into<String> + Send>(
        &self,
        user_id: Uuid,
        name: T,
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            "UPDATE users 
SET name = $1, updated_at = NOW()
WHERE id = $2
RETURNING id, name, username, email, bio, password, created_at, updated_at",
            name.into(),
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    async fn update_user_password(
        &self,
        user_id: Uuid,
        new_password: String,
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            "UPDATE users 
SET password = $1, updated_at = NOW()
WHERE id = $2
RETURNING id, name, username, email, bio, password, created_at, updated_at",
            new_password,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    async fn create_post<T: Into<String> + Send>(
        &self,
        author_id: Uuid,
        title: T,
        content: T,
    ) -> Result<Post, sqlx::Error> {
        let post = sqlx::query_as!(
            Post,
            r#"
        INSERT INTO posts (author_id, title, content)
        VALUES ($1, $2, $3)
        RETURNING
            author_id,
            id,
            title,
            content,
            created_at,
            updated_at
        "#,
            author_id,
            title.into(),
            content.into()
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(post)
    }

    async fn like_post(&self, user_id: Uuid, post_id: Uuid) -> Result<Like, sqlx::Error> {
        let like = sqlx::query_as!(
            Like,
            r#"
        INSERT INTO likes (user_id, post_id)
        VALUES ($1, $2)
        RETURNING user_id, post_id, created_at, updated_at
        "#,
            user_id,
            post_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(like)
    }

    async fn create_comment<T: Into<String> + Send>(
        &self,
        post_id: Uuid,
        user_id: Uuid,
        content: T,
    ) -> Result<Comment, sqlx::Error> {
        let content_str = content.into();

        let comment = sqlx::query_as!(
            Comment,
            r#"
        INSERT INTO comments (post_id, user_id, content)
        VALUES ($1, $2, $3)
        RETURNING id, post_id, user_id, content, created_at, updated_at
        "#,
            post_id,
            user_id,
            content_str
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(comment)
    }

    async fn get_post(&self, post_id: Uuid) -> Result<Option<Post>, sqlx::Error> {
        let post = sqlx::query_as!(
            Post,
            r#"
        SELECT author_id, id, title, content, created_at, updated_at
        FROM posts
        WHERE id = $1
        "#,
            post_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(post)
    }

    async fn get_user_posts(&self, author_id: Uuid) -> Result<Vec<Post>, sqlx::Error> {
        let posts = sqlx::query_as!(
            Post,
            r#"
        SELECT author_id, id, title, content, created_at, updated_at
        FROM posts
        WHERE author_id = $1
        ORDER BY created_at DESC
        "#,
            author_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(posts)
    }

    async fn get_posts(&self, page: u32, limit: usize) -> Result<Vec<Post>, sqlx::Error> {
        let offset = (page - 1) * limit as u32;
        let posts = sqlx::query_as!(
            Post,
            r#"
        SELECT author_id, id, title, content, created_at, updated_at
        FROM posts
        "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(posts)
    }

    async fn update_post(
        &self,
        post_id: Uuid,
        author_id: Uuid,
        title: &str,
        content: &str,
    ) -> Result<Post, sqlx::Error> {
        let post = sqlx::query_as!(
            Post,
            r#"
        UPDATE posts
        SET
            title = $1,
            content = $2,
            updated_at = NOW()
        WHERE id = $3
          AND author_id = $4
        RETURNING
            author_id,
            id,
            title,
            content,
            created_at,
            updated_at
        "#,
            title,
            content,
            post_id,
            author_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(post)
    }

    async fn delete_post(&self, post_id: Uuid, author_id: Uuid) -> Result<(), sqlx::Error> {
        let result = sqlx::query!(
            r#"
        DELETE FROM posts
        WHERE id = $1
          AND author_id = $2
        "#,
            post_id,
            author_id
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        Ok(())
    }

    async fn get_users(&self, page: u32, limit: u32) -> Result<Vec<User>, sqlx::Error> {
        let offset = (page - 1) * limit;

        let users = sqlx::query_as!(
            User,
            r#"
            SELECT
                id,
                name,
                username,
                email,
                bio,
                password,
                created_at,
                updated_at
            FROM users
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit as i64,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }

    async fn increment_view(&self, post_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
        UPDATE posts
        SET views = views + 1
        WHERE id = $1
        "#,
            post_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
