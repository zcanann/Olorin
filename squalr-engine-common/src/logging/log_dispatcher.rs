use crossbeam_channel::Sender;
use log::Record;
use log4rs::append::Append;

#[derive(Debug)]
pub struct LogDispatcher {
    log_sender: Sender<String>,
}

impl LogDispatcher {
    pub fn new(log_sender: Sender<String>) -> Self {
        LogDispatcher { log_sender }
    }
}

impl Append for LogDispatcher {
    fn append(
        &self,
        record: &Record,
    ) -> anyhow::Result<()> {
        let log_message = format!("[{}] {}\n", record.level(), record.args());

        // Just silently fail -- logging more errors inside a failing logging framework risks infinite loops.
        let _ = self.log_sender.send(log_message);

        return Ok(());
    }

    fn flush(&self) {}
}
