use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::sync::mpsc::Sender;
use std::sync::{PoisonError, MutexGuard, RwLockReadGuard, RwLockWriteGuard};

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

impl From<PoisonError<RwLockReadGuard<'_, HashMap<usize, Sender<()>>>>> for SchedulerError {
    fn from(_error: PoisonError<RwLockReadGuard<'_, HashMap<usize, Sender<()>>>>) -> Self {
        SchedulerError::Poisoned
    }
}

impl From<PoisonError<RwLockWriteGuard<'_, HashMap<usize, Sender<()>>>>> for SchedulerError {
    fn from(_error: PoisonError<RwLockWriteGuard<'_, HashMap<usize, Sender<()>>>>) -> Self {
        SchedulerError::Poisoned
    }
}

impl From<PoisonError<MutexGuard<'_, bool>>> for SchedulerError {
    fn from(_error: PoisonError<MutexGuard<'_, bool>>) -> Self {
        SchedulerError::Poisoned
    }
}
