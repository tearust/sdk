use std::io::Write;
use tea_actorx::println;

/// Base logging setting configuration
pub fn set_logging(file: bool, timestamp: bool) {
	let config = tracing_subscriber::fmt()
		.with_writer(|| Logger)
		.with_file(file)
		.with_target(file)
		.with_line_number(file);

	_ = if timestamp {
		config.try_init()
	} else {
		config.without_time().try_init()
	};
}

struct Logger;

impl Write for Logger {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		println!("{}", String::from_utf8_lossy(buf));
		Ok(buf.len())
	}

	fn flush(&mut self) -> std::io::Result<()> {
		Ok(())
	}
}
