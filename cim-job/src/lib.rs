use async_trait::async_trait;

use cim_slo::Result;

#[async_trait]
pub trait Job {
    async fn run(&self) -> Result<()>;
}

#[async_trait]
impl<F> Job for F
where
    F: Fn() -> Result<()> + Sync,
{
    async fn run(&self) -> Result<()> {
        (self)()
    }
}

pub trait Trigger {
    fn next(self, now: i64) -> Result<i64>;
}

pub trait Scheduler {
    fn start() -> Result<()>;
    fn schedule<J: Job, T: Trigger>(&self, job: J, trigger: T) -> Result<()>;
    fn get_keys(&self) -> Vec<String>;
    fn get_job(&self, key: &str) -> Option<()>;
    fn delete(&self, key: &str) -> Result<()>;
    fn has(&self, key: &str) -> bool;
}

#[cfg(test)]
mod tests {}
