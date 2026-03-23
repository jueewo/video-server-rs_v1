use user_auth::OidcConfig;

/// Detect whether we are running in production mode.
/// Set `RUN_MODE=production` in your environment / `.env` to activate.
pub fn is_production() -> bool {
    std::env::var("RUN_MODE")
        .map(|v| v.eq_ignore_ascii_case("production") || v.eq_ignore_ascii_case("prod"))
        .unwrap_or(false)
}

/// Validate that all security-critical configuration is safe for production.
/// Panics (fail-fast) if any check fails — the server must not start with
/// insecure defaults in production.
pub fn validate_production_config(oidc_config: &OidcConfig) {
    let mut errors: Vec<String> = Vec::new();

    // ── RTMP Publish Token ──────────────────────────────────────────
    #[cfg(feature = "media")]
    {
        let rtmp_token = video_manager::rtmp_publish_token();
        if rtmp_token == "supersecret123" || rtmp_token.is_empty() {
            errors.push(
                "RTMP_PUBLISH_TOKEN is missing or still the insecure default 'supersecret123'. \
                 Set a strong, unique token in your environment."
                    .to_string(),
            );
        } else if rtmp_token.len() < 16 {
            errors.push(format!(
                "RTMP_PUBLISH_TOKEN is too short ({} chars). Use at least 16 characters.",
                rtmp_token.len()
            ));
        }
    }

    // ── OIDC Secrets ────────────────────────────────────────────────
    if oidc_config.client_id == "your-client-id" || oidc_config.client_id.is_empty() {
        errors.push(
            "OIDC_CLIENT_ID is missing or still the placeholder 'your-client-id'.".to_string(),
        );
    }
    if oidc_config.client_secret == "your-client-secret" || oidc_config.client_secret.is_empty() {
        errors.push(
            "OIDC_CLIENT_SECRET is missing or still the placeholder 'your-client-secret'."
                .to_string(),
        );
    }

    // ── Session Security ────────────────────────────────────────────
    let session_secure = std::env::var("SESSION_SECURE")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false);
    if !session_secure {
        errors.push("SESSION_SECURE must be 'true' in production (requires HTTPS).".to_string());
    }

    // ── Emergency Login ─────────────────────────────────────────────
    if oidc_config.enable_emergency_login {
        if oidc_config.su_pwd.is_empty() {
            errors.push(
                "ENABLE_EMERGENCY_LOGIN is true but SU_PWD is empty. \
                 Either disable emergency login or set a strong password."
                    .to_string(),
            );
        } else if oidc_config.su_pwd.len() < 12 {
            errors.push(format!(
                "SU_PWD is too short ({} chars). Use at least 12 characters when emergency login is enabled.",
                oidc_config.su_pwd.len()
            ));
        }
        if oidc_config.su_user == "admin" {
            errors.push(
                "SU_USER is still the default 'admin'. Use a non-obvious username in production."
                    .to_string(),
            );
        }
    }

    // ── DATABASE_URL ────────────────────────────────────────────────
    let db_url = std::env::var("DATABASE_URL").unwrap_or_default();
    if db_url.is_empty() {
        errors.push("DATABASE_URL is not set. Explicitly configure the database path.".to_string());
    }

    // ── Fail fast ───────────────────────────────────────────────────
    if !errors.is_empty() {
        eprintln!("\n\u{2554}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2557}");
        eprintln!("\u{2551}  \u{1f6d1}  PRODUCTION STARTUP BLOCKED \u{2014} INSECURE CONFIGURATION     \u{2551}");
        eprintln!("\u{255a}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{255d}\n");
        for (i, err) in errors.iter().enumerate() {
            eprintln!("  {}. {}", i + 1, err);
        }
        eprintln!("\nSet RUN_MODE=development to bypass these checks (NOT for production).\n");
        std::process::exit(1);
    }
}
