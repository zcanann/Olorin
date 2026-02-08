use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Default)]
pub struct ScanExecutionContext {
    cancellation_token: Option<Arc<AtomicBool>>,
    progress_reporter: Option<Arc<dyn Fn(f32) + Send + Sync>>,
}

impl ScanExecutionContext {
    pub fn new(
        cancellation_token: Option<Arc<AtomicBool>>,
        progress_reporter: Option<Arc<dyn Fn(f32) + Send + Sync>>,
    ) -> Self {
        Self {
            cancellation_token,
            progress_reporter,
        }
    }

    pub fn should_cancel(&self) -> bool {
        self.cancellation_token
            .as_ref()
            .map(|cancellation_token| cancellation_token.load(Ordering::SeqCst))
            .unwrap_or(false)
    }

    pub fn report_progress(
        &self,
        progress: f32,
    ) {
        if let Some(progress_reporter) = &self.progress_reporter {
            progress_reporter(progress);
        }
    }
}
