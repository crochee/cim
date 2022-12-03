use http::Request;
use tower_http::trace::MakeSpan;
use tracing::{Level, Span};

#[derive(Clone, Copy)]
pub struct MakeSpanWithTrace {
    level: Level,
    include_headers: bool,
}

impl MakeSpanWithTrace {
    /// Create a new `DefaultMakeSpan`.
    pub fn new() -> Self {
        Self {
            level: Level::DEBUG,
            include_headers: false,
        }
    }

    /// Set the [`Level`] used for the [tracing span].
    ///
    /// Defaults to [`Level::DEBUG`].
    ///
    /// [tracing span]: https://docs.rs/tracing/latest/tracing/#spans
    #[allow(dead_code)]
    pub fn level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    /// Include request headers on the [`Span`].
    ///
    /// By default headers are not included.
    ///
    /// [`Span`]: tracing::Span
    #[allow(dead_code)]
    pub fn include_headers(mut self, include_headers: bool) -> Self {
        self.include_headers = include_headers;
        self
    }
}

impl Default for MakeSpanWithTrace {
    fn default() -> Self {
        Self::new()
    }
}

impl<B> MakeSpan<B> for MakeSpanWithTrace {
    fn make_span(&mut self, request: &Request<B>) -> Span {
        // 判断请求头里是否有trace_id
        let unique_id = match request.headers().get("X-Trace-Id") {
            Some(v) => v.to_str().unwrap_or_default(),
            None => "",
        };
        // This ugly macro is needed, unfortunately, because `tracing::span!`
        // required the level argument to be static. Meaning we can't just pass
        // `self.level`.
        macro_rules! make_span {
            ($level:expr) => {
                if self.include_headers {
                    tracing::span!(
                        $level,
                        "request",
                        trace_id = %unique_id,
                        method = %request.method(),
                        uri = %request.uri(),
                        version = ?request.version(),
                        headers = ?request.headers(),
                    )
                } else {
                    tracing::span!(
                        $level,
                        "request",
                        trace_id = %unique_id,
                        method = %request.method(),
                        uri = %request.uri(),
                        version = ?request.version(),
                    )
                }
            }
        }

        match self.level {
            Level::ERROR => {
                make_span!(Level::ERROR)
            }
            Level::WARN => {
                make_span!(Level::WARN)
            }
            Level::INFO => {
                make_span!(Level::INFO)
            }
            Level::DEBUG => {
                make_span!(Level::DEBUG)
            }
            Level::TRACE => {
                make_span!(Level::TRACE)
            }
        }
    }
}
