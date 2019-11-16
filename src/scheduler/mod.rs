// Copyright (c) Peter Sanders.
// Date: 2019-11-16.
//
// This file defines the public API to be exposed by the scheduler module.
// For the documentation of the respective names, see the modules from which
// they are re-exported.

mod errors;
pub use errors::SchedulerError;

mod scheduler;
pub use scheduler::TaskScheduler;
