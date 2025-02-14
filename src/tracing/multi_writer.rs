mod line_printer;

use std::fs;

use crate::env::system_default_log_dir;

use line_printer::print_log_line;
use tracing_appender::rolling::RollingFileAppender;
use tracing_appender::rolling::Rotation;
use tracing_subscriber::fmt::MakeWriter;

/// Maximum number of log file that must be kept in disk
/// If more than this number, platform will erase them
///
static MAX_LOG_FILES: usize = 3;

///
///
///
pub struct MultiWriter {
    enable_stdout: bool,
    enable_broker_log: bool,
    debug: bool,
    trace: bool,
    filea: tracing_appender::rolling::RollingFileAppender,
}

impl MultiWriter {
    pub fn new(enable_stdout: bool, enable_broker_log: bool, debug: bool, trace: bool) -> Self {
        //
        // Because the appender wait 1 day before pruning
        // without this, developpers will reboot the platform often and won't prune
        Self::prune_old_logs(MAX_LOG_FILES);

        let p = RollingFileAppender::builder()
            .rotation(Rotation::HOURLY) // rotate log files once every day
            .filename_prefix("platform") // log file names will be prefixed
            .filename_suffix("csv") // log file names will be suffixed with `.log`
            .max_log_files(MAX_LOG_FILES)
            .build(system_default_log_dir().unwrap())
            .unwrap();

        Self {
            enable_stdout: enable_stdout,
            enable_broker_log: enable_broker_log,
            debug: debug,
            trace: trace,
            filea: p,
        }
    }

    fn prune_old_logs(max_files: usize) {
        let files = fs::read_dir(system_default_log_dir().unwrap()).map(|dir| {
            dir.filter_map(|entry| {
                let entry = entry.ok()?;
                let metadata = entry.metadata().ok()?;

                // the appender only creates files, not directories or symlinks,
                // so we should never delete a dir or symlink.
                if !metadata.is_file() {
                    return None;
                }

                let filename = entry.file_name();
                // if the filename is not a UTF-8 string, skip it.
                let filename = filename.to_str()?;
                // if let Some(prefix) = &self.log_filename_prefix {
                if !filename.starts_with("platform") {
                    return None;
                }
                // }

                // if let Some(suffix) = &self.log_filename_suffix {
                if !filename.ends_with("csv") {
                    return None;
                }
                // }

                // if self.log_filename_prefix.is_none()
                //     && self.log_filename_suffix.is_none()
                //     && Date::parse(filename, &self.date_format).is_err()
                // {
                //     return None;
                // }

                let created = metadata.created().ok()?;
                Some((entry, created))
            })
            .collect::<Vec<_>>()
        });

        let mut files = match files {
            Ok(files) => files,
            Err(error) => {
                eprintln!("Error reading the log directory/files: {}", error);
                return;
            }
        };
        if files.len() < max_files {
            return;
        }

        // sort the files by their creation timestamps.
        files.sort_by_key(|(_, created_at)| *created_at);

        // delete files, so that (n-1) files remain, because we will create another log file
        for (file, _) in files.iter().take(files.len() - (max_files - 1)) {
            if let Err(error) = fs::remove_file(file.path()) {
                eprintln!(
                    "Failed to remove old log file {}: {}",
                    file.path().display(),
                    error
                );
            }
        }
    }
}

impl std::io::Write for MultiWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        //
        // Store in log file
        self.filea.make_writer().write_all(buf).unwrap();

        //
        // Stdout logs ?
        if self.enable_stdout {
            print_log_line(buf, self.enable_broker_log, self.debug, self.trace);
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.filea.make_writer().flush().unwrap();
        Ok(())
    }
}
