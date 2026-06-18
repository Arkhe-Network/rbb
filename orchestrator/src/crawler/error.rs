use std::fmt;

#[derive(Debug)]
pub enum CrawlerError {
    Http(String),
    Serialization(String),
    Config(String),
    Processing(String),
    Attestation(String),
    Signature(String),
    ZvecError(String),
    Embedding(String),
    RobotsTxtDisallowed,
    RateLimited,
    MaxPagesReached,
}

impl fmt::Display for CrawlerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CrawlerError::Http(msg) => write!(f, "HTTP error: {}", msg),
            CrawlerError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            CrawlerError::Config(msg) => write!(f, "Configuration error: {}", msg),
            CrawlerError::Processing(msg) => write!(f, "Processing error: {}", msg),
            CrawlerError::Attestation(msg) => write!(f, "Attestation error: {}", msg),
            CrawlerError::Signature(msg) => write!(f, "Signature error: {}", msg),
            CrawlerError::ZvecError(msg) => write!(f, "zVEC error: {}", msg),
            CrawlerError::Embedding(msg) => write!(f, "Embedding error: {}", msg),
            CrawlerError::RobotsTxtDisallowed => write!(f, "Robots.txt disallowed"),
            CrawlerError::RateLimited => write!(f, "Rate limited"),
            CrawlerError::MaxPagesReached => write!(f, "Max pages reached"),
        }
    }
}

impl std::error::Error for CrawlerError {}

pub type Result<T> = std::result::Result<T, CrawlerError>;
