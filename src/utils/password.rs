use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

use crate::error::{ErrorMessage, HttpError};

const MAX_PASSWORD: usize = 64;

pub fn hash_password(password: impl Into<String>) -> Result<String, HttpError> {
    let password = password.into();

    if password.is_empty() {
        return Err(HttpError::bad_request(
            ErrorMessage::EmptyPassword.to_string(),
        ));
    }

    if password.len() > MAX_PASSWORD {
        return Err(HttpError::bad_request(
            ErrorMessage::ExceededMaxPasswordLength(MAX_PASSWORD).to_string(),
        ));
    }

    let salt = SaltString::generate(&mut OsRng);
    let hash_password = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| HttpError::bad_request(ErrorMessage::HashingError.to_string()))?
        .to_string();

    Ok(hash_password)
}

pub fn compare_password(hashed_password: &str, password: &str) -> Result<bool, HttpError> {
    if password.is_empty() {
        return Err(HttpError::bad_request(
            ErrorMessage::EmptyPassword.to_string(),
        ));
    }

    if password.len() > MAX_PASSWORD {
        return Err(HttpError::bad_request(
            ErrorMessage::ExceededMaxPasswordLength(MAX_PASSWORD).to_string(),
        ));
    }

    let parsed_hash =
        PasswordHash::new(hashed_password).map_err(|_| HttpError::bad_request(ErrorMessage::InvalidHashFormat.to_string()))?;

    let password_verify = Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_or(Ok(false), |_| Ok(true))?;

    Ok(password_verify)
}
