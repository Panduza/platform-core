mod csv_formatter;
mod logger;
mod multi_writer;

pub use logger::AttributeLogger;
pub use logger::DriverLogger;
pub use logger::FactoryLogger;
pub use logger::InstanceLogger;
pub use logger::Logger;

use csv_formatter::CSVFormatter;
use multi_writer::MultiWriter;
use tracing_appender::non_blocking::WorkerGuard;

/// Keep the non_blocking writer guard alive as long as the application is alive
/// Else writer stop working
///
static mut GUARD: Option<WorkerGuard> = None;

/// Function to initiliaze tracing for the application
///
pub fn init(enable_stdout: bool, enable_broker_log: bool, debug: bool, trace: bool) {
    let multiw = MultiWriter::new(enable_stdout, enable_broker_log, debug, trace);

    let (non_blocking, guard) = tracing_appender::non_blocking(multiw);

    unsafe {
        GUARD = Some(guard);
    }

    let subscriber = tracing_subscriber::fmt()
        // .with_max_level(tracing::Level::TRACE)
        .with_max_level(tracing::Level::TRACE)
        // Display source code file paths
        .with_file(false)
        // Display source code line numbers
        .with_line_number(false)
        // Display the thread ID an event was recorded on
        .with_thread_ids(true)
        // Build the subscriber
        .event_format(CSVFormatter {})
        // Custom writer
        .with_writer(non_blocking)
        // Ok
        .finish();
    // use that subscriber to process traces emitted after this point
    tracing::subscriber::set_global_default(subscriber).unwrap();
}
