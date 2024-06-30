use std::ops::Range;
use std::sync::Arc;

use egui::text::LayoutJob;
use egui::{
	Align, Button, CollapsingHeader, FontSelection, RichText, ScrollArea, TextEdit, TextStyle, Ui,
	Vec2,
};
use nucleo::pattern::{CaseMatching, Normalization};
use nucleo::{Config, Nucleo};
use sandbox_sdk::reflection::ReflectionFunction;
use sandbox_sdk::{UFunction, UObject};

use super::render_type;

pub struct FunctionViewer {
	nucleo: Nucleo<String>,
	search: String,
	function: Option<ReflectionFunction>,
}
impl FunctionViewer {
	pub fn new() -> Self {
		let nucleo = Nucleo::new(Config::DEFAULT, Arc::new(|| {}), None, 1);
		let injector = nucleo.injector();

		for func in FunctionViewer::functions() {
			injector.push(func, |f, col| col[0] = f.as_str().into());
		}

		FunctionViewer {
			nucleo,
			search: String::new(),
			function: None,
		}
	}

	pub fn ui(&mut self, ui: &mut Ui) {
		let search = ui.add(
			TextEdit::singleline(&mut self.search)
				.font(TextStyle::Monospace)
				.desired_width(f32::INFINITY),
		);

		if search.changed() {
			self.nucleo.pattern.reparse(
				0,
				&self.search,
				CaseMatching::Smart,
				Normalization::Smart,
				false,
			);
		}

		if search.gained_focus() {
			self.function = None;
		}

		ui.separator();

		if let Some(function) = &self.function {
			ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
				FunctionViewer::render_function(ui, &function);
			});
		} else {
			self.nucleo.tick(10);

			let snap = self.nucleo.snapshot();

			let row_height = ui.text_style_height(&TextStyle::Monospace) + 6.0;
			let total_rows = snap.matched_item_count();

			let scroll = ScrollArea::vertical().auto_shrink(false);

			let scroll = if search.changed() {
				scroll.vertical_scroll_offset(0.0)
			} else {
				scroll
			};

			scroll.show_rows(
				ui,
				row_height,
				total_rows as usize,
				|ui, Range { start, end }| {
					let row_range = Range {
						start: start as u32,
						end: end as u32,
					};

					for matched in snap.matched_items(row_range) {
						let function = format!("Function {}", matched.data);

						let btn = Button::new(RichText::new(&function).monospace())
							.min_size(Vec2::new(ui.available_width(), 0.0))
							.wrap(false)
							.frame(false);

						let btn = ui.add(btn);

						if btn.clicked() {
							if let Some(function) = UFunction::FindFunction(&function) {
								self.function = Some(ReflectionFunction::new(function));
							}
						}

						ui.separator();
					}
				},
			);
		}
	}

	fn functions() -> Vec<String> {
		UObject::GObjObjects()
			.iter()
			.flatten()
			.filter_map(|obj| obj.Cast::<UFunction>())
			.map(|function| function.GetPathName())
			.collect()
	}

	fn render_function(ui: &mut Ui, function: &ReflectionFunction) {
		ui.label(format!("Function {}", function.name));

		CollapsingHeader::new("Params")
			.default_open(true)
			.show(ui, |ui| {
				for param in &function.params {
					let mut job = LayoutJob::default();
					let style = ui.style();

					RichText::new(&param.prop.name).monospace().append_to(
						&mut job,
						&style,
						FontSelection::Default,
						Align::Center,
					);
					RichText::new(": ").monospace().append_to(
						&mut job,
						&style,
						FontSelection::Default,
						Align::Center,
					);

					render_type(&param.prop.uetype, &mut job);

					ui.label(job);
				}
			});

		if let Some(ret_val) = &function.ret {
			CollapsingHeader::new("Return Value")
				.default_open(true)
				.show(ui, |ui| {
					let mut job = LayoutJob::default();

					render_type(&ret_val.param.prop.uetype, &mut job);

					ui.label(job);
				});
		}
	}
}
