use std::error::Error;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;

#[derive(Debug)]
pub enum SchedulerError {
    AlreadyRunning,
    Poisoned,
    NoSuchTask,
    ConsumerDied,
}

impl Error for SchedulerError {}

impl Display for SchedulerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SchedulerError::AlreadyRunning => {
                write!(f, "scheduler was already running")
            },
            SchedulerError::Poisoned => {
                write!(f, "mutex poisoned")
            },
            SchedulerError::NoSuchTask => {
                write!(f, "trying to delete nonexistent task")
            },
            SchedulerError::ConsumerDied => {
                write!(f, "task thread died unexpectedly")
            }
        }
    }
}
