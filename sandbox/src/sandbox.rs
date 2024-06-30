use std::collections::BTreeMap;
use std::ffi::CString;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use color_eyre::eyre::Context as _;
use color_eyre::Result;
use egui::epaint::Shadow;
use egui::{Context, FontData, FontDefinitions, FontFamily, Key, Margin, Rounding, Style};
use egui_aesthetix::Aesthetix;
use egui_dx11_internal::Dx11App;
use sandbox_sdk::{StaticClass, UFunction, UObject};
use tracing::{error, info, info_span, trace, trace_span};
use windows::core::PCSTR;
use windows::Win32::Foundation::HMODULE;
use windows::Win32::System::LibraryLoader::FreeLibraryAndExitThread;
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxA, MB_ICONERROR, MB_OK};

use crate::devtools::Devtools;
use crate::dx11hooks::Dx11Hooks;
use crate::logging::Logging;
use crate::tracing::SandboxTracing;

pub struct Sandbox {
	open: bool,
	logging: Logging,
	devtools: Devtools,
	exit: Arc<AtomicBool>,
}

impl Sandbox {
	/// Sandbox entry point
	///
	/// will attempt initialization and try to fail gracefully without
	/// ever crashing rocket league
	pub fn main(module: HMODULE) -> ! {
		// Initialize tracing as early as possible
		let tracing = match SandboxTracing::init() {
			Ok(tracing) => tracing,
			Err(err) => Sandbox::fail(module, err),
		};

		// Initialize color_eyre hooks to get tracing for errors
		if let Err(err) = color_eyre::install() {
			Sandbox::fail(module, err.wrap_err("Failed to install color_eyre hooks"));
		};

		// Create the root trace and enter
		info_span!("main").in_scope(|| {
			info!("Initialized tracing");

			let (exit, sandbox) = match Sandbox::new(tracing) {
				Ok(it) => it,
				Err(err) => Sandbox::fail(module, err),
			};

			info!("Initialized Sandbox");

			let app = Dx11App::with_mut_context(move |ctx, sb| sb.ui(ctx), sandbox, Sandbox::theme);

			let dx11_hooks = match Dx11Hooks::hook(app) {
				Ok(it) => it,
				Err(err) => Sandbox::fail(module, err.wrap_err("Failed to initialize DX11 hooks")),
			};

			trace!("Hooked DirectX11");

			while !exit.load(Ordering::Relaxed) {
				thread::sleep(Duration::from_millis(10));
			}

			// Unhook dx11
			drop(dx11_hooks);

			trace!("Unhooked DirectX11");
		});

		Sandbox::exit(module);
	}

	fn new(tracing: Arc<SandboxTracing>) -> Result<(Arc<AtomicBool>, Self)> {
		Sandbox::init_sdk();

		let devtools = Devtools::new().context("Failed to create dev tools")?;
		let logging = Logging::new(tracing);

		let exit = Arc::new(AtomicBool::new(false));

		Ok((
			Arc::clone(&exit),
			Sandbox {
				open: true,
				devtools,
				logging,
				exit,
			},
		))
	}

	fn init_sdk() {
		trace_span!("ue_sdk_init").in_scope(|| {
			let _ = UObject::GObjObjects();
			trace!("Initialized globals");

			let _ = UObject::StaticClass();
			trace!("Initialized FindClass cache");

			let _ = UFunction::FindFunction("");
			trace!("Initialized FindFunction cache");
		});
	}

	fn theme(ctx: &mut Context) {
		let mut style = egui_aesthetix::themes::CarlDark.custom_style();
		let default = Style::default();

		style.interaction = default.interaction;
		style.spacing = default.spacing;
		style.spacing.menu_margin = Margin::symmetric(4.0, 3.0);

		style.visuals.collapsing_header_frame = false;
		style.visuals.window_shadow = Shadow::NONE;
		style.visuals.resize_corner_size = 6.0;
		style.visuals.window_rounding = Rounding::same(5.0);
		style.visuals.clip_rect_margin = 3.0;

		ctx.set_style(style);

		let mut font_data = BTreeMap::new();
		let mut families = BTreeMap::new();

		font_data.insert(
			String::from("JetBrains"),
			FontData::from_static(include_bytes!("../assets/fonts/JetBrainsMono.ttf")),
		);

		font_data.insert(
			String::from("SourceSans"),
			FontData::from_static(include_bytes!("../assets/fonts/SourceSans3.ttf")),
		);

		families.insert(FontFamily::Monospace, vec![String::from("JetBrains")]);

		families.insert(FontFamily::Proportional, vec![String::from("SourceSans")]);

		ctx.set_fonts(FontDefinitions {
			font_data,
			families,
		});
	}

	fn ui(&mut self, ctx: &Context) {
		info_span!("sandbox_ui").in_scope(|| {
			egui::Window::new("Sandbox")
				.open(&mut self.open)
				.default_size((400.0, 250.0))
				.show(ctx, |ui| {
					ui.label("Welcome to Sandbox");

					ui.allocate_space(ui.available_size());
				});

			self.logging.ui(ctx);
			self.devtools.ui(ctx);

			if ctx.input(|i| i.key_pressed(Key::Home)) {
				self.open = !self.open;
			}

			if ctx.input(|i| i.key_pressed(Key::End)) {
				trace!("End pressed, exiting");
				self.exit.store(true, Ordering::Relaxed);
			}
		})
	}

	fn fail(module: HMODULE, err: color_eyre::Report) -> ! {
		error!("Failed to initialize sandbox: {err}");

		let err = CString::new(err.to_string()).unwrap_or(c"Failed to format error".to_owned());

		unsafe { MessageBoxA(None, PCSTR(err.as_ptr().cast()), None, MB_ICONERROR | MB_OK) };

		Sandbox::exit(module);
	}

	fn exit(module: HMODULE) -> ! {
		thread::sleep(Duration::from_secs(1));
		unsafe { FreeLibraryAndExitThread(module, 0) }
	}
}
