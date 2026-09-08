//! Background job queue and task manager (LP-0905).
//!
//! Provides bounded worker execution, priorities, cancellation tokens,
//! and lifecycle tracking for asynchronous background tasks.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum JobPriority {
    Low = 0,
    Normal = 1,
    High = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSummary {
    pub id: String,
    pub name: String,
    pub priority: JobPriority,
    pub status: JobStatus,
    pub progress_percent: u8,
    pub error: Option<String>,
}

pub struct CancellationToken {
    cancelled: AtomicBool,
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            cancelled: AtomicBool::new(false),
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

pub struct JobEntry {
    pub summary: JobSummary,
    pub token: Arc<CancellationToken>,
    pub created_at: Instant,
}

pub struct BackgroundJobManager {
    jobs: Mutex<HashMap<String, JobEntry>>,
    max_history: usize,
}

impl BackgroundJobManager {
    pub fn new(max_history: usize) -> Self {
        Self {
            jobs: Mutex::new(HashMap::new()),
            max_history,
        }
    }

    pub fn submit_job(&self, name: String, priority: JobPriority) -> (String, Arc<CancellationToken>) {
        let id = Uuid::new_v4().to_string();
        let token = Arc::new(CancellationToken::new());
        let summary = JobSummary {
            id: id.clone(),
            name,
            priority,
            status: JobStatus::Queued,
            progress_percent: 0,
            error: None,
        };

        let mut lock = self.jobs.lock().expect("jobs mutex poisoned");
        if lock.len() >= self.max_history {
            // Prune completed or cancelled jobs older than remaining
            let mut finished_ids: Vec<String> = lock
                .iter()
                .filter(|(_, j)| matches!(j.summary.status, JobStatus::Completed | JobStatus::Cancelled | JobStatus::Failed))
                .map(|(id, _)| id.clone())
                .collect();
            finished_ids.truncate(lock.len() - self.max_history + 1);
            for fid in finished_ids {
                lock.remove(&fid);
            }
        }

        lock.insert(
            id.clone(),
            JobEntry {
                summary,
                token: Arc::clone(&token),
                created_at: Instant::now(),
            },
        );

        (id, token)
    }

    pub fn update_progress(&self, id: &str, progress: u8) {
        let mut lock = self.jobs.lock().expect("jobs mutex poisoned");
        if let Some(job) = lock.get_mut(id) {
            job.summary.status = JobStatus::Running;
            job.summary.progress_percent = progress.min(100);
        }
    }

    pub fn complete_job(&self, id: &str) {
        let mut lock = self.jobs.lock().expect("jobs mutex poisoned");
        if let Some(job) = lock.get_mut(id) {
            job.summary.status = JobStatus::Completed;
            job.summary.progress_percent = 100;
        }
    }

    pub fn fail_job(&self, id: &str, err: String) {
        let mut lock = self.jobs.lock().expect("jobs mutex poisoned");
        if let Some(job) = lock.get_mut(id) {
            job.summary.status = JobStatus::Failed;
            job.summary.error = Some(err);
        }
    }

    pub fn cancel_job(&self, id: &str) -> bool {
        let mut lock = self.jobs.lock().expect("jobs mutex poisoned");
        if let Some(job) = lock.get_mut(id) {
            job.token.cancel();
            job.summary.status = JobStatus::Cancelled;
            true
        } else {
            false
        }
    }

    pub fn get_job(&self, id: &str) -> Option<JobSummary> {
        let lock = self.jobs.lock().expect("jobs mutex poisoned");
        lock.get(id).map(|j| j.summary.clone())
    }

    pub fn list_jobs(&self) -> Vec<JobSummary> {
        let lock = self.jobs.lock().expect("jobs mutex poisoned");
        let mut list: Vec<JobSummary> = lock.values().map(|j| j.summary.clone()).collect();
        list.sort_by_key(|j| std::cmp::Reverse(j.priority));
        list
    }
}

impl Default for BackgroundJobManager {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn submit_track_and_complete_job() {
        let manager = BackgroundJobManager::new(10);
        let (id, _token) = manager.submit_job("collection_run".into(), JobPriority::High);

        assert_eq!(manager.get_job(&id).unwrap().status, JobStatus::Queued);

        manager.update_progress(&id, 50);
        let job = manager.get_job(&id).unwrap();
        assert_eq!(job.status, JobStatus::Running);
        assert_eq!(job.progress_percent, 50);

        manager.complete_job(&id);
        let completed = manager.get_job(&id).unwrap();
        assert_eq!(completed.status, JobStatus::Completed);
        assert_eq!(completed.progress_percent, 100);
    }

    #[test]
    fn cancel_running_job() {
        let manager = BackgroundJobManager::new(10);
        let (id, token) = manager.submit_job("sync_repo".into(), JobPriority::Normal);

        assert!(!token.is_cancelled());
        assert!(manager.cancel_job(&id));
        assert!(token.is_cancelled());
        assert_eq!(manager.get_job(&id).unwrap().status, JobStatus::Cancelled);
    }

    #[test]
    fn priorities_are_ordered_descending() {
        let manager = BackgroundJobManager::new(10);
        manager.submit_job("low_task".into(), JobPriority::Low);
        manager.submit_job("high_task".into(), JobPriority::High);
        manager.submit_job("normal_task".into(), JobPriority::Normal);

        let list = manager.list_jobs();
        assert_eq!(list[0].priority, JobPriority::High);
        assert_eq!(list[1].priority, JobPriority::Normal);
        assert_eq!(list[2].priority, JobPriority::Low);
    }
}
