//! Rate limiting middleware for the media server (TD-010)
//!
//! Provides configurable per-IP rate limiting for different endpoint classes:
//! - **Auth**: login, OIDC, emergency auth — strict (brute-force protection)
//! - **Upload**: media upload — moderate (resource protection)
//! - **Validation**: access codes, stream tokens — moderate (abuse prevention)
//! - **ApiMutate**: create/update/delete operations — moderate
//! - **General**: all other endpoints — lenient
//!
//! # Configuration
//!
//! All limits are configurable via environment variables:
//! ```text
//! RATE_LIMIT_AUTH_RPM=10          # requests per minute per IP
//! RATE_LIMIT_AUTH_BURST=5         # burst allowance
//! RATE_LIMIT_UPLOAD_RPM=15
//! RATE_LIMIT_UPLOAD_BURST=5
//! RATE_LIMIT_VALIDATION_RPM=20
//! RATE_LIMIT_VALIDATION_BURST=10
//! RATE_LIMIT_API_MUTATE_RPM=30
//! RATE_LIMIT_API_MUTATE_BURST=10
//! RATE_LIMIT_GENERAL_RPM=120
//! RATE_LIMIT_GENERAL_BURST=30
//! RATE_LIMIT_ENABLED=true         # master switch (default: true)
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! use rate_limiter::RateLimitConfig;
//!
//! let rl = RateLimitConfig::from_env();
//!
//! // Apply to route groups — each returns Option<RateLimitLayer> (None if disabled)
//! let app = Router::new()
//!     .merge(if let Some(layer) = rl.auth_layer() {
//!         auth_routes(state).layer(layer)
//!     } else {
//!         auth_routes(state)
//!     });
//! ```

use governor::middleware::NoOpMiddleware;
use std::sync::Arc;

use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::SmartIpKeyExtractor;
pub use tower_governor::GovernorLayer;

/// The concrete GovernorLayer type used throughout this crate.
///
/// - `SmartIpKeyExtractor`: checks X-Forwarded-For → X-Real-IP → Forwarded → ConnectInfo → peer
/// - `NoOpMiddleware`: no extra rate-limit state headers (keeps responses clean)
pub type RateLimitLayer = GovernorLayer<SmartIpKeyExtractor, NoOpMiddleware, axum::body::Body>;

// ---------------------------------------------------------------------------
// Endpoint classes
// ---------------------------------------------------------------------------

/// Logical endpoint class — each gets its own rate limit policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointClass {
    /// Login, OIDC, emergency auth — strictest limits
    Auth,
    /// Media upload — moderate limits
    Upload,
    /// Access-code validation, stream-token validation
    Validation,
    /// API create / update / delete operations
    ApiMutate,
    /// Default for all other endpoints
    General,
}

impl std::fmt::Display for EndpointClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auth => write!(f, "auth"),
            Self::Upload => write!(f, "upload"),
            Self::Validation => write!(f, "validation"),
            Self::ApiMutate => write!(f, "api_mutate"),
            Self::General => write!(f, "general"),
        }
    }
}

// ---------------------------------------------------------------------------
// Per-class limit parameters
// ---------------------------------------------------------------------------

/// Rate limit parameters for a single endpoint class.
#[derive(Debug, Clone, Copy)]
pub struct ClassLimit {
    /// Requests allowed per minute per IP.
    pub requests_per_minute: u64,
    /// Maximum burst size above the sustained rate.
    pub burst: u32,
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Full rate-limit configuration for the application.
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Master switch — if false, all limiter helpers return `None` (pass-through).
    pub enabled: bool,
    pub auth: ClassLimit,
    pub upload: ClassLimit,
    pub validation: ClassLimit,
    pub api_mutate: ClassLimit,
    pub general: ClassLimit,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auth: ClassLimit {
                requests_per_minute: 10,
                burst: 5,
            },
            upload: ClassLimit {
                requests_per_minute: 15,
                burst: 5,
            },
            validation: ClassLimit {
                requests_per_minute: 20,
                burst: 10,
            },
            api_mutate: ClassLimit {
                requests_per_minute: 30,
                burst: 10,
            },
            general: ClassLimit {
                requests_per_minute: 120,
                burst: 30,
            },
        }
    }
}

impl RateLimitConfig {
    /// Load configuration from environment variables, falling back to defaults.
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Master switch
        if let Ok(v) = std::env::var("RATE_LIMIT_ENABLED") {
            config.enabled = matches!(v.to_lowercase().as_str(), "true" | "1" | "yes");
        }

        // Auth
        if let Ok(v) = std::env::var("RATE_LIMIT_AUTH_RPM") {
            if let Ok(n) = v.parse() {
                config.auth.requests_per_minute = n;
            }
        }
        if let Ok(v) = std::env::var("RATE_LIMIT_AUTH_BURST") {
            if let Ok(n) = v.parse() {
                config.auth.burst = n;
            }
        }

        // Upload
        if let Ok(v) = std::env::var("RATE_LIMIT_UPLOAD_RPM") {
            if let Ok(n) = v.parse() {
                config.upload.requests_per_minute = n;
            }
        }
        if let Ok(v) = std::env::var("RATE_LIMIT_UPLOAD_BURST") {
            if let Ok(n) = v.parse() {
                config.upload.burst = n;
            }
        }

        // Validation
        if let Ok(v) = std::env::var("RATE_LIMIT_VALIDATION_RPM") {
            if let Ok(n) = v.parse() {
                config.validation.requests_per_minute = n;
            }
        }
        if let Ok(v) = std::env::var("RATE_LIMIT_VALIDATION_BURST") {
            if let Ok(n) = v.parse() {
                config.validation.burst = n;
            }
        }

        // API Mutate
        if let Ok(v) = std::env::var("RATE_LIMIT_API_MUTATE_RPM") {
            if let Ok(n) = v.parse() {
                config.api_mutate.requests_per_minute = n;
            }
        }
        if let Ok(v) = std::env::var("RATE_LIMIT_API_MUTATE_BURST") {
            if let Ok(n) = v.parse() {
                config.api_mutate.burst = n;
            }
        }

        // General
        if let Ok(v) = std::env::var("RATE_LIMIT_GENERAL_RPM") {
            if let Ok(n) = v.parse() {
                config.general.requests_per_minute = n;
            }
        }
        if let Ok(v) = std::env::var("RATE_LIMIT_GENERAL_BURST") {
            if let Ok(n) = v.parse() {
                config.general.burst = n;
            }
        }

        config
    }

    /// Create a [`RateLimitLayer`] for the given endpoint class.
    ///
    /// Returns `None` if rate limiting is disabled via `RATE_LIMIT_ENABLED=false`.
    pub fn layer_for(&self, class: EndpointClass) -> Option<RateLimitLayer> {
        if !self.enabled {
            return None;
        }

        let limit = match class {
            EndpointClass::Auth => self.auth,
            EndpointClass::Upload => self.upload,
            EndpointClass::Validation => self.validation,
            EndpointClass::ApiMutate => self.api_mutate,
            EndpointClass::General => self.general,
        };

        build_layer(limit, class)
    }

    // ── Convenience helpers ─────────────────────────────────────────

    /// Auth rate-limiter layer (login, OIDC, emergency auth).
    pub fn auth_layer(&self) -> Option<RateLimitLayer> {
        self.layer_for(EndpointClass::Auth)
    }

    /// Upload rate-limiter layer.
    pub fn upload_layer(&self) -> Option<RateLimitLayer> {
        self.layer_for(EndpointClass::Upload)
    }

    /// Validation rate-limiter layer (access codes, stream tokens).
    pub fn validation_layer(&self) -> Option<RateLimitLayer> {
        self.layer_for(EndpointClass::Validation)
    }

    /// API mutation rate-limiter layer (create/update/delete).
    pub fn api_mutate_layer(&self) -> Option<RateLimitLayer> {
        self.layer_for(EndpointClass::ApiMutate)
    }

    /// General / default rate-limiter layer.
    pub fn general_layer(&self) -> Option<RateLimitLayer> {
        self.layer_for(EndpointClass::General)
    }

    /// Print configuration summary to stdout (matches the server startup style).
    pub fn print_summary(&self) {
        if !self.enabled {
            println!("⚡ Rate Limiting: DISABLED (RATE_LIMIT_ENABLED=false)");
            return;
        }
        println!("⚡ Rate Limiting: ENABLED");
        println!(
            "   - Auth:       {} rpm, burst {}",
            self.auth.requests_per_minute, self.auth.burst
        );
        println!(
            "   - Upload:     {} rpm, burst {}",
            self.upload.requests_per_minute, self.upload.burst
        );
        println!(
            "   - Validation: {} rpm, burst {}",
            self.validation.requests_per_minute, self.validation.burst
        );
        println!(
            "   - API Mutate: {} rpm, burst {}",
            self.api_mutate.requests_per_minute, self.api_mutate.burst
        );
        println!(
            "   - General:    {} rpm, burst {}",
            self.general.requests_per_minute, self.general.burst
        );
    }
}

// ---------------------------------------------------------------------------
// Governor layer factory
// ---------------------------------------------------------------------------

/// Build a [`RateLimitLayer`] from per-class parameters.
///
/// The governor `period` is the replenishment interval for a single token.
/// We convert from RPM: `period = 60_000ms / RPM` (minimum 1ms).
fn build_layer(limit: ClassLimit, class: EndpointClass) -> Option<RateLimitLayer> {
    let rpm = std::cmp::max(1, limit.requests_per_minute);
    let burst = std::cmp::max(1, limit.burst);

    // Governor replenishes one token per `period`.
    // period = 60_000ms / RPM → e.g. 10 RPM = one token every 6 000ms
    let period_ms = std::cmp::max(1, 60_000 / rpm);

    let config = GovernorConfigBuilder::default()
        .key_extractor(SmartIpKeyExtractor)
        .per_millisecond(period_ms)
        .burst_size(burst)
        .finish()
        .unwrap_or_else(|| {
            panic!(
                "rate-limiter: failed to build governor config for {} (period_ms={}, burst={})",
                class, period_ms, burst
            );
        });

    tracing::debug!(
        class = %class,
        rpm = rpm,
        period_ms = period_ms,
        burst = burst,
        "rate limiter configured"
    );

    Some(GovernorLayer::new(Arc::new(config)))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_sane() {
        let config = RateLimitConfig::default();
        assert!(config.enabled);
        assert_eq!(config.auth.requests_per_minute, 10);
        assert_eq!(config.auth.burst, 5);
        assert_eq!(config.upload.requests_per_minute, 15);
        assert_eq!(config.general.requests_per_minute, 120);
    }

    #[test]
    fn disabled_config_returns_none() {
        let mut config = RateLimitConfig::default();
        config.enabled = false;
        assert!(config.auth_layer().is_none());
        assert!(config.upload_layer().is_none());
        assert!(config.general_layer().is_none());
    }

    #[test]
    fn enabled_config_returns_some() {
        let config = RateLimitConfig::default();
        assert!(config.auth_layer().is_some());
        assert!(config.upload_layer().is_some());
        assert!(config.validation_layer().is_some());
        assert!(config.api_mutate_layer().is_some());
        assert!(config.general_layer().is_some());
    }

    #[test]
    fn endpoint_class_display() {
        assert_eq!(EndpointClass::Auth.to_string(), "auth");
        assert_eq!(EndpointClass::Upload.to_string(), "upload");
        assert_eq!(EndpointClass::Validation.to_string(), "validation");
        assert_eq!(EndpointClass::ApiMutate.to_string(), "api_mutate");
        assert_eq!(EndpointClass::General.to_string(), "general");
    }

    #[test]
    fn build_layer_works() {
        let limit = ClassLimit {
            requests_per_minute: 60,
            burst: 10,
        };
        let layer = build_layer(limit, EndpointClass::General);
        assert!(layer.is_some());
    }

    #[test]
    fn build_layer_low_rpm() {
        // Even very low RPM should work (clamped to minimum 1)
        let limit = ClassLimit {
            requests_per_minute: 1,
            burst: 1,
        };
        let layer = build_layer(limit, EndpointClass::Auth);
        assert!(layer.is_some());
    }

    #[test]
    fn build_layer_high_rpm() {
        let limit = ClassLimit {
            requests_per_minute: 10_000,
            burst: 100,
        };
        let layer = build_layer(limit, EndpointClass::General);
        assert!(layer.is_some());
    }

    #[test]
    fn env_override_works() {
        // Set env vars for this test
        std::env::set_var("RATE_LIMIT_AUTH_RPM", "42");
        std::env::set_var("RATE_LIMIT_AUTH_BURST", "7");
        std::env::set_var("RATE_LIMIT_ENABLED", "true");

        let config = RateLimitConfig::from_env();
        assert_eq!(config.auth.requests_per_minute, 42);
        assert_eq!(config.auth.burst, 7);
        assert!(config.enabled);

        // Clean up
        std::env::remove_var("RATE_LIMIT_AUTH_RPM");
        std::env::remove_var("RATE_LIMIT_AUTH_BURST");
        std::env::remove_var("RATE_LIMIT_ENABLED");
    }

    #[test]
    fn disabled_via_env() {
        std::env::set_var("RATE_LIMIT_ENABLED", "false");
        let config = RateLimitConfig::from_env();
        assert!(!config.enabled);
        assert!(config.auth_layer().is_none());
        std::env::remove_var("RATE_LIMIT_ENABLED");
    }
}
