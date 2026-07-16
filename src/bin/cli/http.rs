use reqwest::{Client, Error, Method, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::{env, fmt, io};

use crate::token;

const BASE_URL_ENV: &str = "API_BASE_URL";
const DEFAULT_BASE_URL: &str = "http://localhost:3000";

pub enum CliError {
    NotLoggedIn,
    Request(Error),
    Api { status: StatusCode, body: String },
    Io(io::Error),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::NotLoggedIn => write!(f, "not logged in - run 'cli login' first"),
            CliError::Request(e) => write!(f, "request failed {e}"),
            CliError::Api { status, body } => write!(f, "{status}: {body}"),
            CliError::Io(e) => write!(f, "io error: {e}"),
        }
    }
}

impl From<Error> for CliError {
    fn from(e: Error) -> Self {
        CliError::Request(e)
    }
}

impl From<io::Error> for CliError {
    fn from(e: io::Error) -> Self {
        CliError::Io(e)
    }
}

fn base_url() -> String {
    env::var(BASE_URL_ENV).unwrap_or_else(|_| DEFAULT_BASE_URL.to_string())
}

pub async fn request<T: DeserializeOwned>(
    method: Method,
    path: &str,
    auth: bool,
    body: Option<Value>,
) -> Result<T, CliError> {
    let url = format!("{}{}", base_url(), path);
    let mut builder = Client::new().request(method, url);

    if auth {
        let token = token::load_token().ok_or(CliError::NotLoggedIn)?;
        builder = builder.bearer_auth(token);
    }

    if let Some(body) = body {
        builder = builder.json(&body);
    }

    let response = builder.send().await?;
    let status = response.status();
    let text = response.text().await?;

    if !status.is_success() {
        return Err(CliError::Api { status, body: text });
    }

    serde_json::from_str(&text).map_err(|e| CliError::Api {
        status,
        body: format!("invalid response JSON: {e} (raw body: {text})"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_logged_in_message() {
        assert_eq!(
            CliError::NotLoggedIn.to_string(),
            "not logged in - run 'cli login' first"
        );
    }

    #[test]
    fn api_error_includes_status_and_body() {
        let err = CliError::Api {
            status: StatusCode::CONFLICT,
            body: "username already taken".to_string(),
        };
        assert_eq!(err.to_string(), "409 Conflict: username already taken");
    }

    #[test]
    fn io_error_includes_underlying_message() {
        let err = CliError::Io(io::Error::new(io::ErrorKind::NotFound, "no such file"));
        assert_eq!(err.to_string(), "io error: no such file");
    }
}
