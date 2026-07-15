use std::time::Duration;

use reqwest::{Method, header::HeaderMap};

pub const MAX_ATTEMPTS: usize = 3;
const MAX_DELAY: Duration = Duration::from_secs(30);
const BASE_DELAY: Duration = Duration::from_millis(250);

pub fn delay_for(
    method: &Method,
    status: Option<u16>,
    headers: Option<&HeaderMap>,
    attempt: usize,
) -> Option<Duration> {
    if attempt >= MAX_ATTEMPTS || !is_read_method(method) {
        return None;
    }

    if let Some(status) = status
        && !matches!(status, 429 | 502 | 503 | 504)
    {
        return None;
    }

    let retry_after = headers
        .and_then(|headers| headers.get("retry-after"))
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_secs);

    Some(
        retry_after
            .unwrap_or_else(|| exponential_delay(attempt))
            .min(MAX_DELAY),
    )
}

fn is_read_method(method: &Method) -> bool {
    matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS)
}

fn exponential_delay(attempt: usize) -> Duration {
    let multiplier = 1u32 << attempt.saturating_sub(1).min(7);

    BASE_DELAY.saturating_mul(multiplier)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retries_read_transient_status() {
        let delay = delay_for(&Method::GET, Some(503), None, 1);

        assert_eq!(delay, Some(Duration::from_millis(250)));
    }

    #[test]
    fn test_honors_retry_after_and_caps_delay() {
        let mut headers = HeaderMap::new();
        headers.insert("retry-after", "45".parse().unwrap());

        let delay = delay_for(&Method::GET, Some(429), Some(&headers), 1);

        assert_eq!(delay, Some(Duration::from_secs(30)));
    }

    #[test]
    fn test_does_not_retry_mutations() {
        let delay = delay_for(&Method::POST, Some(503), None, 1);

        assert!(delay.is_none());
    }

    #[test]
    fn test_stops_after_max_attempts() {
        let delay = delay_for(&Method::GET, Some(503), None, MAX_ATTEMPTS);

        assert!(delay.is_none());
    }
}
