use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use uuid::Uuid;

use crate::{
    models::user::{Credentials, User},
    state::AppState,
};

impl AppState {
    pub async fn insert_user(&self, credentials: Credentials) -> Result<User, sqlx::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(credentials.password.as_bytes(), &salt)
            .expect("hashing a password should not fail")
            .to_string();

        let id = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO users (id, username, password_hash) VALUES (?, ?, ?)",
            id,
            credentials.username,
            password_hash
        )
        .execute(&self.pool)
        .await?;

        Ok(User {
            id,
            username: credentials.username,
        })
    }
    pub async fn verify_credentials(
        &self,
        credentials: Credentials,
    ) -> Result<Option<Uuid>, sqlx::Error> {
        let row = sqlx::query!(
            r#"SELECT id as "id: Uuid", password_hash FROM users WHERE username = ?"#,
            credentials.username
        )
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let parsed_hash =
            PasswordHash::new(&row.password_hash).expect("stored hash should be valid PHC string");
        if Argon2::default()
            .verify_password(credentials.password.as_bytes(), &parsed_hash)
            .is_ok()
        {
            Ok(Some(row.id))
        } else {
            Ok(None)
        }
    }
}
