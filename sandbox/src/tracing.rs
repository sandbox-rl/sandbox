use std::fs::OpenOptions;
use std::io;
use std::ops::Deref;
use std::sync::Arc;

use color_eyre::eyre::Context;
use color_eyre::Result;
use parking_lot::RwLock;
use tracing_error::ErrorLayer;
use tracing_subscriber::layer::SubscriberExt;

#[derive(Default)]
pub struct SandboxTracing {
	inner: RwLock<Vec<u8>>,
}

impl SandboxTracing {
	pub fn init() -> Result<Arc<Self>> {
		let writer = SandboxTracing::default();
		let writer = Arc::new(writer);

		SandboxTracing::init_logging(Arc::clone(&writer))?;

		Ok(writer)
	}

	pub fn output(&self) -> impl Deref<Target = Vec<u8>> + '_ {
		self.inner.read()
	}

	fn init_logging(writer: Arc<SandboxTracing>) -> Result<()> {
		let timer = tracing_subscriber::fmt::time::ChronoLocal::new(String::from("%k:%M:%S%.3f"));

		let egui_layer = tracing_subscriber::fmt::layer()
			.with_file(false)
			.with_line_number(false)
			.with_target(false)
			.with_timer(timer)
			.with_writer(writer);

		let logfile = OpenOptions::new()
			.append(true)
			.create(true)
			.open("Z:\\home\\avalsch\\sandbox.log")
			.context("Failed to open log file")?;

		let file_layer = tracing_subscriber::fmt::layer()
			.with_target(true)
			.with_thread_names(true)
			.with_writer(logfile);

		let subscriber = tracing_subscriber::Registry::default()
			.with(egui_layer)
			.with(file_layer)
			.with(ErrorLayer::default());

		tracing::subscriber::set_global_default(subscriber)?;

		Ok(())
	}
}

impl<'a> io::Write for &'a SandboxTracing {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		self.inner.write().write(buf)
	}

	fn flush(&mut self) -> io::Result<()> {
		self.inner.write().flush()
	}
}
