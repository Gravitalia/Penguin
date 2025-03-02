use rand::distributions::{Alphanumeric, DistString};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

const TOKEN_LENGTH: usize = 64;

/// Database user representation.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    #[serde(skip)]
    pub email: String,
    pub avatar: Option<String>,
    pub flags: i32,
    #[serde(skip)]
    pub(crate) password: String,
    pub created_at: chrono::NaiveDate,
    #[sqlx(json)]
    pub public_keys: Vec<Key>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Key {
    pub id: String,
    pub owner: String,
    pub public_key_pem: String,
    pub created_at: chrono::NaiveDate,
}

impl User {
    /// Update `vanity` of en empty [`User`].
    /// Do not work for fetched users.
    pub fn with_id(mut self, user_id: String) -> Self {
        self.id = user_id;
        self
    }

    /// Update `email` of an empty [`User`].
    /// Do not work for fetched users.
    pub fn with_email(mut self, email: String) -> Self {
        self.email = email;
        self
    }

    /// Get data on a user.
    pub async fn get(self, conn: &Pool<Postgres>) -> Result<Self, sqlx::Error> {
        if !self.id.is_empty() {
            sqlx::query_as::<_, User>(
                r#"SELECT 
                    u.id,
                    u.username,
                    u.email,
                    u.avatar,
                    u.flags,
                    u.password,
                    u.created_at,
                    CASE
                        WHEN COUNT(k.id) = 0 THEN '[]'
                        ELSE JSONB_AGG(
                            jsonb_build_object(
                                'id', cast(k.id as TEXT),
                                'owner', k.user_id,
                                'public_key_pem', k.key,
                                'created_at', k.created_at
                            )
                        )
                    END AS public_keys
                FROM users u
                LEFT JOIN keys k ON k.user_id = u.id
                WHERE u.id = $1
                GROUP BY 
                    u.id, 
                    u.username, 
                    u.email, 
                    u.avatar, 
                    u.flags, 
                    u.password, 
                    u.created_at;
                "#
            )
            .bind(self.id)
            .fetch_one(conn)
            .await
        } else if !self.email.is_empty() {
            sqlx::query_as::<_, User>(
                r#"SELECT
                    u.id,
                    u.username,
                    u.email,
                    u.avatar,
                    u.flags,
                    u.password,
                    u.created_at,
                    CASE
                        WHEN COUNT(k.id) = 0 THEN '[]'
                        ELSE JSONB_AGG(
                            jsonb_build_object(
                                'id', cast(k.id as TEXT),
                                'owner', k.user_id,
                                'public_key_pem', k.key,
                                'created_at', k.created_at
                            )
                        )
                    END AS public_keys
                FROM users u
                LEFT JOIN keys k ON k.user_id = u.id
                WHERE u.email = $1
                GROUP BY 
                    u.id, 
                    u.username, 
                    u.email, 
                    u.avatar, 
                    u.flags,
                    u.password, 
                    u.created_at;
                "#
            )
            .bind(self.email)
            .fetch_one(conn)
            .await
        } else {
            Err(sqlx::Error::ColumnNotFound(
                "Missing column 'id' or 'email' column".to_owned(),
            ))
        }
    }

    /// Generate a token for this specific user.
    pub async fn generate_token(&self, conn: &Pool<Postgres>) -> Result<String, sqlx::Error> {
        if self.id.is_empty() {
            return Err(sqlx::Error::ColumnNotFound(
                "Missing column 'id' column".into(),
            ));
        }

        let token = Alphanumeric.sample_string(&mut OsRng, TOKEN_LENGTH);

        sqlx::query!(
            r#"INSERT INTO "tokens" (token, user_id) values ($1, $2)"#,
            token,
            self.id,
        )
        .execute(conn)
        .await?;

        Ok(token)
    }
}
