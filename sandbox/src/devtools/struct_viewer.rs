use std::ops::Range;
use std::sync::Arc;

use egui::text::LayoutJob;
use egui::{
	Align, Button, CollapsingHeader, FontSelection, RichText, ScrollArea, TextEdit, TextStyle, Ui,
	Vec2,
};
use nucleo::pattern::{CaseMatching, Normalization};
use nucleo::{Config, Nucleo};
use sandbox_sdk::reflection::ReflectionStruct;
use sandbox_sdk::{UObject, UScriptStruct};

use super::render_type;

pub struct StructViewer {
	nucleo: Nucleo<String>,
	search: String,
	strct: Option<ReflectionStruct>,
}

impl StructViewer {
	pub fn new() -> Self {
		let nucleo = Nucleo::new(Config::DEFAULT, Arc::new(|| {}), None, 1);
		let injector = nucleo.injector();

		for strct in StructViewer::structs() {
			injector.push(strct, |c, col| col[0] = c.as_str().into());
		}

		StructViewer {
			nucleo,
			search: String::new(),
			strct: None,
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
			self.strct = None;
		}

		ui.separator();

		if let Some(strct) = &self.strct {
			ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
				StructViewer::render_struct(ui, &strct);
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
						let strct = format!("ScriptStruct {}", matched.data);

						let btn = Button::new(RichText::new(&strct).monospace())
							.min_size(Vec2::new(ui.available_width(), 0.0))
							.wrap(false)
							.frame(false);

						let btn = ui.add(btn);

						if btn.clicked() {
							if let Some(strct) = UScriptStruct::FindStruct(&strct) {
								self.strct = Some(ReflectionStruct::new(strct));
							}
						}

						ui.separator();
					}
				},
			);
		}
	}

	pub fn render_struct(ui: &mut Ui, strct: &ReflectionStruct) {
		CollapsingHeader::new(RichText::new(strct.strct.GetFullName()).monospace())
			.default_open(true)
			.show(ui, |ui| {
				if !strct.properties.is_empty() {
					ui.set_min_width(ui.available_width());
					for prop in &strct.properties {
						let mut job = LayoutJob::default();
						let style = ui.style();

						RichText::new(&prop.name).monospace().append_to(
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
						render_type(&prop.uetype, &mut job);

						ui.label(job);
					}
				}
			});
	}

	fn structs() -> Vec<String> {
		UObject::GObjObjects()
			.iter()
			.flatten()
			.filter_map(|obj| obj.Cast::<UScriptStruct>())
			.map(|strct| strct.GetPathName())
			.collect()
	}
}
