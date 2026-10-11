use anyhow::{Context, Result};

#[derive(Clone, Debug)]
pub struct Config {
    pub ami_host: String,
    pub ami_port: u16,
    pub ami_user: String,
    pub ami_pass: String,
    pub ws_bind: String,

    pub database_url: String,
    pub jwt_secret: String,
    pub access_token_ttl_minutes: i64,
    pub refresh_token_ttl_days: i64,
    pub seed_dev_users: bool,

    pub app_env: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            ami_host: std::env::var("AMI_HOST").unwrap_or_else(|_| "127.0.0.1".into()),
            ami_port: std::env::var("AMI_PORT")
                .unwrap_or_else(|_| "5038".into())
                .parse()
                .context("AMI_PORT must be a number")?,
            ami_user: std::env::var("AMI_USER").context("AMI_USER missing")?,
            ami_pass: std::env::var("AMI_PASS").context("AMI_PASS missing")?,
            ws_bind: std::env::var("WS_BIND").unwrap_or_else(|_| "0.0.0.0:8080".into()),

            database_url: std::env::var("DATABASE_URL").context("DATABASE_URL missing")?,
            jwt_secret: std::env::var("JWT_SECRET").context("JWT_SECRET missing")?,
            access_token_ttl_minutes: std::env::var("ACCESS_TOKEN_TTL_MINUTES")
                .unwrap_or_else(|_| "15".into())
                .parse()
                .context("ACCESS_TOKEN_TTL_MINUTES must be a number")?,
            refresh_token_ttl_days: std::env::var("REFRESH_TOKEN_TTL_DAYS")
                .unwrap_or_else(|_| "7".into())
                .parse()
                .context("REFRESH_TOKEN_TTL_DAYS must be a number")?,
            seed_dev_users: std::env::var("SEED_DEV_USERS")
                .unwrap_or_else(|_| "false".into())
                .to_lowercase()
                == "true",

            app_env: std::env::var("APP_ENV").unwrap_or_else(|_| "production".into()),
        })
    }

    pub fn is_development(&self) -> bool {
        self.app_env.eq_ignore_ascii_case("development")
    }

    /// Refuse to start if dev secrets are still in use outside of development.
    ///
    /// A "dev secret" is any value that shipped in the repository or in
    /// documentation. If we detect one and APP_ENV is not "development",
    /// the process exits with a clear message instead of starting insecure.
    pub fn validate_secrets(&self) -> Result<()> {
        let known_dev_secrets: &[&str] = &[
            "changeme_strong_password",
            "callcenter_dev_password",
            "dev_only_change_me_to_a_long_random_string_at_least_32_chars",
            "replace_me",
        ];

        let mut problems: Vec<&str> = Vec::new();

        if known_dev_secrets.contains(&self.jwt_secret.as_str()) {
            problems.push("JWT_SECRET");
        }
        if known_dev_secrets.contains(&self.ami_pass.as_str()) {
            problems.push("AMI_PASS");
        }
        if self.jwt_secret.len() < 32 {
            problems.push("JWT_SECRET (must be at least 32 characters)");
        }
        for secret in known_dev_secrets {
            if self.database_url.contains(secret) {
                problems.push("DATABASE_URL password");
                break;
            }
        }

        if problems.is_empty() {
            return Ok(());
        }

        if self.is_development() {
            tracing::warn!(
                "Running with DEVELOPMENT secrets. Not safe for production. \
                 Please rotate: {}",
                problems.join(", ")
            );
            return Ok(());
        }

        anyhow::bail!(
            "Refusing to start with default/weak secrets: {}. \
             Rotate them or set APP_ENV=development to allow (not for production).",
            problems.join(", ")
        );
    }
}
