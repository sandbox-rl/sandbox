use std::convert::identity;
use std::sync::Arc;

use cansi::v3::{categorise_text, CategorisedSlice};
use cansi::Color;
use egui::text::LayoutJob;
use egui::{Align, Color32, Context, FontSelection, Key, RichText, ScrollArea, Window};

use crate::theme::{BLACK, BLUE, CYAN, GREEN, MAGENTA, RED, WHITE, YELLOW};
use crate::tracing::SandboxTracing;

pub struct Logging {
	open: bool,
	tracing: Arc<SandboxTracing>,
}

impl Logging {
	pub fn new(tracing: Arc<SandboxTracing>) -> Self {
		Logging {
			open: true,
			tracing,
		}
	}

	pub fn logs(&self) -> Vec<egui::RichText> {
		let logs = self.tracing.output();

		let logs = String::from_utf8_lossy(&logs);
		let logs = categorise_text(&logs);

		logs.into_iter()
			.map(convert_cansi_categorized_to_rich_text)
			.collect()
	}

	pub fn ui(&mut self, ctx: &Context) {
		let logs = self.logs();

		Window::new("Sandbox Logs")
			.default_size((700.0, 450.0))
			.min_size((400.0, 200.0))
			.max_size((1000.0, 1000.0))
			.open(&mut self.open)
			.show(ctx, |ui| {
				ScrollArea::vertical()
					.auto_shrink(false)
					.stick_to_bottom(true)
					.show(ui, move |ui| {
						let mut job = LayoutJob::default();

						for text in &logs {
							text.clone().append_to(
								&mut job,
								&ctx.style(),
								FontSelection::Default,
								Align::Center,
							);
						}

						ui.label(job);
					});
			});

		if ctx.input(|i| i.key_pressed(Key::F1)) {
			self.open = !self.open;
		}
	}
}

fn convert_cansi_categorized_to_rich_text(
	CategorisedSlice {
		text,
		fg,
		bg,
		italic,
		underline,
		strikethrough,
		..
	}: CategorisedSlice,
) -> RichText {
	let fg = fg.map(convert_cansi_egui_color);
	let bg = bg.map(convert_cansi_egui_color);

	let mut text = RichText::new(text).monospace().size(14.0);

	if let Some(fg) = fg {
		text = text.color(fg);
	}

	if let Some(bg) = bg {
		text = text.background_color(bg);
	}

	if italic.is_some_and(identity) {
		text = text.italics();
	}

	if underline.is_some_and(identity) {
		text = text.underline();
	}

	if strikethrough.is_some_and(identity) {
		text = text.strikethrough()
	}

	text
}

fn convert_cansi_egui_color(color: Color) -> Color32 {
	match color {
		Color::Black | Color::BrightBlack => BLACK,
		Color::White | Color::BrightWhite => WHITE,
		Color::Red | Color::BrightRed => RED,
		Color::Yellow | Color::BrightYellow => YELLOW,
		Color::Green | Color::BrightGreen => GREEN,
		Color::Cyan | Color::BrightCyan => CYAN,
		Color::Blue | Color::BrightBlue => BLUE,
		Color::Magenta | Color::BrightMagenta => MAGENTA,
	}
}
