mod class_viewer;
mod function_viewer;
mod struct_viewer;

use color_eyre::Result;
use egui::text::LayoutJob;
use egui::{Align, FontSelection, Key, RichText, Style, Ui, Window};
use egui_dock::{DockArea, DockState, TabViewer};
use sandbox_sdk::reflection::{PointerProperty, PropertyType, StructProperty, TemplateProperty};

use self::class_viewer::ClassViewer;
use self::function_viewer::FunctionViewer;
use self::struct_viewer::StructViewer;
use crate::theme::{BLUE, CYAN, LIGHT_GREEN, MAGENTA, PURPLE, RED, WHITE};

pub enum Tabs {
	ClassViewer,
	StructViewer,
	FunctionViewer,
}

pub struct Devtools {
	open: bool,
	dock_state: DockState<Tabs>,
	tab_viewer: DevtoolsTabViewer,
}

impl Devtools {
	pub fn new() -> Result<Self> {
		let dock_state = DockState::new(vec![
			Tabs::ClassViewer,
			Tabs::FunctionViewer,
			Tabs::StructViewer,
		]);
		let tab_viewer = DevtoolsTabViewer::new();

		Ok(Self {
			open: false,
			dock_state,
			tab_viewer,
		})
	}

	pub fn ui(&mut self, ctx: &egui::Context) {
		Window::new("Sandbox Developer Tools")
			.open(&mut self.open)
			.default_size((800.0, 450.0))
			.min_size((650.0, 400.0))
			.show(ctx, |ui| {
				let style = &**ui.style();
				ui.set_style(Style {
					override_text_style: Some(egui::TextStyle::Monospace),
					..style.clone()
				});

				DockArea::new(&mut self.dock_state).show_inside(ui, &mut self.tab_viewer);
			});

		if ctx.input(|i| i.key_pressed(Key::F5)) {
			self.open = !self.open;
		}
	}
}

struct DevtoolsTabViewer {
	class_viewer: ClassViewer,
	struct_viewer: StructViewer,
	function_viewer: FunctionViewer,
}

impl DevtoolsTabViewer {
	fn new() -> Self {
		let class_viewer = ClassViewer::new();
		let struct_viewer = StructViewer::new();
		let function_viewer = FunctionViewer::new();

		Self {
			class_viewer,
			struct_viewer,
			function_viewer,
		}
	}
}

impl TabViewer for DevtoolsTabViewer {
	type Tab = Tabs;

	fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
		match tab {
			Tabs::ClassViewer => "Class Viewer".into(),
			Tabs::StructViewer => "Struct Viewer".into(),
			Tabs::FunctionViewer => "Function Viewer".into(),
		}
	}

	fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
		match tab {
			Tabs::ClassViewer => self.class_viewer.ui(ui),
			Tabs::StructViewer => self.struct_viewer.ui(ui),
			Tabs::FunctionViewer => self.function_viewer.ui(ui),
		}
	}

	fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool {
		false
	}

	fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
		false
	}

	fn scroll_bars(&self, _tab: &Self::Tab) -> [bool; 2] {
		[false; 2]
	}
}

fn render_type(prop: &PropertyType, mut job: &mut LayoutJob) {
	let append_to_job = |text: RichText, job: &mut LayoutJob| {
		text.append_to(
			job,
			&Style::default(),
			FontSelection::Default,
			Align::Center,
		)
	};

	match prop {
		PropertyType::Native(_) => {
			append_to_job(RichText::new(prop.type_name()).monospace().color(BLUE), job)
		}
		PropertyType::Struct(
			StructProperty::FName | StructProperty::FString | StructProperty::FScriptDelegate,
		) => append_to_job(RichText::new(prop.type_name()).monospace().color(CYAN), job),

		PropertyType::Struct(StructProperty::FStruct(name)) => {
			append_to_job(RichText::new(name).monospace().color(CYAN), job)
		}
		PropertyType::Pointer(
			PointerProperty::UObject(name)
			| PointerProperty::UClass(name)
			| PointerProperty::UInterface(name),
		) => {
			append_to_job(RichText::new("*").monospace().color(WHITE), job);
			append_to_job(RichText::new("mut ").monospace().color(MAGENTA), job);
			append_to_job(RichText::new(name).monospace().color(PURPLE), job);
		}
		PropertyType::Template(TemplateProperty::TArray(inner)) => {
			append_to_job(RichText::new("TArray").monospace().color(LIGHT_GREEN), job);
			append_to_job(RichText::new("<").monospace().color(WHITE), job);
			render_type(&inner, job);
			append_to_job(RichText::new(">").monospace().color(WHITE), job);
		}
		PropertyType::Template(TemplateProperty::TMap { key, val }) => {
			append_to_job(RichText::new("TMap").monospace().color(LIGHT_GREEN), job);
			append_to_job(RichText::new("<").monospace().color(WHITE), job);
			render_type(&key, job);
			append_to_job(RichText::new(",").monospace().color(WHITE), job);
			render_type(&val, &mut job);
			append_to_job(RichText::new(">").monospace().color(WHITE), job);
		}
		PropertyType::Unknown => {
			append_to_job(RichText::new("Unknown").monospace().color(RED), job)
		}
	}
}
