use std::sync::atomic::{AtomicBool, Ordering};

/// Cheap catch-up progress; stored summaries remain the source of truth for actual jobs.
pub struct EvaluationWork {
    pending: AtomicBool,
    changed: AtomicBool,
}

impl Default for EvaluationWork {
    fn default() -> Self {
        Self {
            pending: AtomicBool::new(true),
            changed: AtomicBool::new(false),
        }
    }
}

impl EvaluationWork {
    pub fn summary_changed(&self) {
        self.changed.store(true, Ordering::SeqCst);
    }

    pub fn take_change(&self) -> bool {
        if !self.changed.load(Ordering::SeqCst) {
            return false;
        }
        self.pending.store(true, Ordering::SeqCst);
        self.changed.swap(false, Ordering::SeqCst)
    }

    pub fn report_pending(&self, pending: bool) {
        self.pending.store(pending, Ordering::SeqCst);
    }

    pub fn has_pending(&self) -> bool {
        self.pending.load(Ordering::SeqCst) || self.changed.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catch_up_waits_for_first_scan_and_for_replacements() {
        let work = EvaluationWork::default();
        assert!(work.has_pending());
        work.report_pending(false);
        assert!(!work.has_pending());
        work.summary_changed();
        // A scan that finishes after the replacement cannot erase that new work.
        work.report_pending(false);
        assert!(work.has_pending());
        assert!(work.take_change());
        assert!(!work.take_change());
        work.report_pending(true);
        assert!(work.has_pending());
    }
}
