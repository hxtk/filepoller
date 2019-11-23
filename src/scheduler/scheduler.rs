use std::collections::HashMap;
use std::result::Result;
use std::sync::mpsc;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use super::errors;

/// TaskScheduler is a scheduler for tasks that should be run repeatedly at regular
/// intervals.
pub struct TaskScheduler {
    running: Arc<Mutex<bool>>,
    tasks: Arc<RwLock<HashMap<usize, mpsc::Sender<()>>>>,
}

impl TaskScheduler {
    /// The default constructor produces a TaskScheduler with no jobs scheduled and
    /// in a stopped state.
    pub fn new() -> Self {
        TaskScheduler {
            running: Arc::new(Mutex::new(false)),
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Schedule a task to run at a given interval.
    ///
    /// The task will not be executed unless the scheduler is in a running state, i.e.,
    /// another thread has already called `run`.
    ///
    /// On success, this function will return the ID of the task, which can be used to
    /// cancel it later; otherwise an `errors::SchedulerError::Poisoned` will be returned
    /// if another thread has died while adding or removing a task.
    pub fn add_task(
        &self,
        frequency: Duration,
        job: impl Fn() + Send + Sync + 'static,
    ) -> Result<usize, errors::SchedulerError> {
        let mut tasks = self.tasks.write()?;

        let task_id: usize = match tasks.keys().max() {
            Some(x) => x + 1,
            None => 0,
        };

        let (tx, rx) = mpsc::channel();
        let _task_thread = thread::spawn(move || {
            let job = Arc::new(job);
            let mut last_run = Instant::now() - frequency;
            loop {
                match rx.recv() {
                    Ok(_) => {
                        if last_run.elapsed() >= frequency {
                            last_run = Instant::now();
                            let task = job.clone();
                            thread::spawn(move || task());
                        }
                    }
                    // This is a normal exit condition that will occur when
                    // the context running the scheduler dies.
                    Err(_) => return,
                }
            }
        });

        tasks.insert(task_id, tx);

        Ok(task_id)
    }

    /// Remove at task by ID.
    ///
    /// This will not halt execution of a running task, but it will prevent
    /// new executions from occurring.
    ///
    /// Possible errors include `errors::SchedulerError::NoSuchtask` if the given
    /// `task_id` is not known to the scheduler and
    /// `errors::scheduler::Poisoned`, which occurs if a
    pub fn remove_task(&self, task_id: usize) -> Result<(), errors::SchedulerError> {
        let mut tasks = self.tasks.write()?;

        match tasks.remove(&task_id) {
            Some(_) => Ok(()),
            None => Err(errors::SchedulerError::NoSuchTask),
        }
    }

    /// Run the scheduled jobs forever or until an error occurs.
    ///
    /// We do not attempt to run jobs exactly at their scheduled intervals.
    /// Tasks will be given an opportunity at every clock tick, which will occur at a
    /// time interval of `resulution`. If `resolution` is larger than the `interval` of
    /// a task then that task will be executed exactly once every `resolution`. In
    /// general, the timing of task execution will never be reliably more precise than
    /// `resolution`, but arbitrarily small `resolution` begins to approximate busy-wait.
    ///
    /// Specifically, for all tasks having `interval`s greater than `resolution`,
    /// for all continuous time ranges of size `interval` during which the scheduler
    /// is in a running state, the corresponding task will start execution exactly once.
    ///
    /// Possible errors include `errors::SchedulerError::Poisoned`, which occurs
    /// when a thread fails while adding a task or while removing a task;
    /// `errors::SchedulerError::ConsumerDied`, which occurs when the thread for a
    /// given task dies for any reason other than the task being removed; and
    /// `errors::SchedulerError::AlreadyRunning` if another thread is already running
    /// the scheduler.
    ///
    /// If an error occurs then this function will return a corresponding `Err`, otherwise
    /// it will return an empty `Ok`.
    pub fn run(&self, resolution: Duration) -> Result<(), errors::SchedulerError> {
        let mut running = self.running.lock()?;
        match *running {
            true => Err(errors::SchedulerError::AlreadyRunning),
            false => {
                *running = true;
                Ok(())
            }
        }?;

        while *self.running.lock()? {
            let tasks = self.tasks.read()?;

            tasks
                .iter()
                .map(|(_, channel)| channel.send(()))
                .fold(Ok(()), |acc, x| match x {
                    Ok(_) => acc,
                    Err(_) => Err(errors::SchedulerError::ConsumerDied),
                })?;

            thread::sleep(resolution);
        }

        Ok(())
    }
}
