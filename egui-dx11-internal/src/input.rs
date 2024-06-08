use std::mem;
use std::time::{SystemTime, UNIX_EPOCH};

use clipboard::windows_clipboard::WindowsClipboardContext;
use clipboard::ClipboardProvider;
use egui::{Event, Key, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::System::SystemServices::{MK_CONTROL, MK_SHIFT};
use windows::Win32::UI::Input::KeyboardAndMouse::{
	GetAsyncKeyState, VIRTUAL_KEY, VK_BACK, VK_CONTROL, VK_DELETE, VK_DOWN, VK_END, VK_ESCAPE,
	VK_HOME, VK_INSERT, VK_LEFT, VK_NEXT, VK_PRIOR, VK_RETURN, VK_RIGHT, VK_SHIFT, VK_SPACE,
	VK_TAB, VK_UP,
};
use windows::Win32::UI::WindowsAndMessaging::{
	GetClientRect, WHEEL_DELTA, WM_CHAR, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_LBUTTONUP,
	WM_MBUTTONDBLCLK, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEHWHEEL, WM_MOUSEMOVE, WM_MOUSEWHEEL,
	WM_NCLBUTTONDBLCLK, WM_RBUTTONDBLCLK, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
	WM_XBUTTONDBLCLK, WM_XBUTTONDOWN, WM_XBUTTONUP, XBUTTON1, XBUTTON2,
};

pub struct InputCollector {
	hwnd: HWND,
	events: Vec<Event>,
	modifiers: Option<Modifiers>,
}

impl InputCollector {
	pub fn collect_input(&mut self) -> RawInput {
		RawInput {
			modifiers: self.modifiers.unwrap_or_default(),
			events: mem::take(&mut self.events),
			screen_rect: Some(self.get_screen_rect()),
			time: Some(Self::get_system_time()),
			..Default::default()
		}
	}

	fn get_screen_rect(&self) -> Rect {
		let mut rect = RECT::default();
		let _ = unsafe { GetClientRect(self.hwnd, &mut rect) };

		let max = Pos2::new(
			(rect.right - rect.left) as f32,
			(rect.bottom - rect.top) as f32,
		);

		Rect {
			min: Pos2::ZERO,
			max,
		}
	}

	fn get_system_time() -> f64 {
		SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.unwrap()
			.as_secs_f64()
	}

	pub fn new(hwnd: HWND) -> Self {
		InputCollector {
			hwnd,
			events: vec![],
			modifiers: None,
		}
	}

	pub fn process(&mut self, umsg: u32, wparam: usize, lparam: isize) {
		match umsg {
			WM_MOUSEMOVE => {
				self.alter_modifiers(get_mouse_modifiers(wparam));
				self.events.push(Event::PointerMoved(get_pos(lparam)));
			}
			WM_LBUTTONDOWN | WM_NCLBUTTONDBLCLK => {
				let modifiers = get_mouse_modifiers(wparam);
				self.alter_modifiers(modifiers);

				self.events.push(Event::PointerButton {
					pos: get_pos(lparam),
					button: PointerButton::Primary,
					pressed: true,
					modifiers,
				})
			}
			WM_LBUTTONUP => {
				let modifiers = get_mouse_modifiers(wparam);
				self.alter_modifiers(modifiers);

				self.events.push(Event::PointerButton {
					pos: get_pos(lparam),
					button: PointerButton::Primary,
					pressed: false,
					modifiers,
				})
			}
			WM_RBUTTONDOWN | WM_RBUTTONDBLCLK => {
				let modifiers = get_mouse_modifiers(wparam);
				self.alter_modifiers(modifiers);

				self.events.push(Event::PointerButton {
					pos: get_pos(lparam),
					button: PointerButton::Secondary,
					pressed: true,
					modifiers,
				});
			}
			WM_RBUTTONUP => {
				let modifiers = get_mouse_modifiers(wparam);
				self.alter_modifiers(modifiers);

				self.events.push(Event::PointerButton {
					pos: get_pos(lparam),
					button: PointerButton::Secondary,
					pressed: false,
					modifiers,
				});
			}
			WM_MBUTTONDOWN | WM_MBUTTONDBLCLK => {
				let modifiers = get_mouse_modifiers(wparam);
				self.alter_modifiers(modifiers);

				self.events.push(Event::PointerButton {
					pos: get_pos(lparam),
					button: PointerButton::Middle,
					pressed: true,
					modifiers,
				});
			}
			WM_MBUTTONUP => {
				let modifiers = get_mouse_modifiers(wparam);
				self.alter_modifiers(modifiers);

				self.events.push(Event::PointerButton {
					pos: get_pos(lparam),
					button: PointerButton::Middle,
					pressed: false,
					modifiers,
				});
			}
			WM_XBUTTONDOWN | WM_XBUTTONDBLCLK => {
				let modifiers = get_mouse_modifiers(wparam);
				self.alter_modifiers(modifiers);

				self.events.push(Event::PointerButton {
					pos: get_pos(lparam),
					button: if (wparam as u32) >> 16 & XBUTTON1 as u32 != 0 {
						PointerButton::Extra1
					} else if (wparam as u32) >> 16 & XBUTTON2 as u32 != 0 {
						PointerButton::Extra2
					} else {
						unreachable!()
					},
					pressed: true,
					modifiers,
				});
			}
			WM_XBUTTONUP => {
				let modifiers = get_mouse_modifiers(wparam);
				self.alter_modifiers(modifiers);

				self.events.push(Event::PointerButton {
					pos: get_pos(lparam),
					button: if (wparam as u32) >> 16 & XBUTTON1 as u32 != 0 {
						PointerButton::Extra1
					} else if (wparam as u32) >> 16 & XBUTTON2 as u32 != 0 {
						PointerButton::Extra2
					} else {
						unreachable!()
					},
					pressed: false,
					modifiers,
				});
			}
			WM_CHAR => {
				if let Some(ch) = char::from_u32(wparam as u32) {
					if !ch.is_control() {
						self.events.push(Event::Text(ch.into()));
					}
				}
			}
			WM_MOUSEWHEEL => {
				self.alter_modifiers(get_mouse_modifiers(wparam));

				let delta = (wparam >> 16) as i16 as f32 * 10. / WHEEL_DELTA as f32;

				if wparam & MK_CONTROL.0 as usize != 0 {
					self.events
						.push(Event::Zoom(if delta > 0. { 1.5 } else { 0.5 }));
				} else {
					self.events.push(Event::Scroll(Vec2::new(0., delta)));
				}
			}
			WM_MOUSEHWHEEL => {
				self.alter_modifiers(get_mouse_modifiers(wparam));

				let delta = (wparam >> 16) as i16 as f32 * 10. / WHEEL_DELTA as f32;

				if wparam & MK_CONTROL.0 as usize != 0 {
					self.events
						.push(Event::Zoom(if delta > 0. { 1.5 } else { 0.5 }));
				} else {
					self.events.push(Event::Scroll(Vec2::new(delta, 0.0)));
				}
			}
			msg @ (WM_KEYDOWN | WM_SYSKEYDOWN) => {
				let modifiers = get_key_modifiers(msg);
				self.modifiers = Some(modifiers);

				if let Some(key) = get_key(wparam) {
					if key == Key::V && modifiers.ctrl {
						if let Some(clipboard) = get_clipboard_text() {
							self.events.push(Event::Text(clipboard));
						}
					}

					if key == Key::C && modifiers.ctrl {
						self.events.push(Event::Copy);
					}

					if key == Key::X && modifiers.ctrl {
						self.events.push(Event::Cut);
					}

					self.events.push(Event::Key {
						key,
						physical_key: None,
						pressed: true,
						repeat: false,
						modifiers,
					});
				}
			}
			msg @ (WM_KEYUP | WM_SYSKEYUP) => {
				let modifiers = get_key_modifiers(msg);
				self.modifiers = Some(modifiers);

				if let Some(key) = get_key(wparam) {
					self.events.push(Event::Key {
						physical_key: None,
						pressed: false,
						repeat: false,
						modifiers,
						key,
					})
				}
			}
			_ => {}
		}
	}

	fn alter_modifiers(&mut self, new: Modifiers) {
		if let Some(old) = self.modifiers.as_mut() {
			*old = new;
		}
	}
}

fn get_pos(lparam: isize) -> Pos2 {
	let x = (lparam & 0xffff) as i16 as f32;
	let y = (lparam >> 16 & 0xffff) as i16 as f32;

	Pos2::new(x, y)
}

fn get_mouse_modifiers(wparam: usize) -> Modifiers {
	Modifiers {
		alt: false,
		ctrl: wparam & MK_CONTROL.0 as usize != 0,
		shift: wparam & MK_SHIFT.0 as usize != 0,
		mac_cmd: false,
		command: wparam & MK_CONTROL.0 as usize != 0,
	}
}

fn get_key_modifiers(msg: u32) -> Modifiers {
	let ctrl = unsafe { GetAsyncKeyState(VK_CONTROL.0 as i32) != 0 };
	let shift = unsafe { GetAsyncKeyState(VK_SHIFT.0 as i32) != 0 };

	Modifiers {
		alt: msg == WM_SYSKEYDOWN,
		ctrl,
		shift,
		mac_cmd: false,
		command: ctrl,
	}
}

fn get_key(wparam: usize) -> Option<Key> {
	match wparam {
		0x30..=0x39 => unsafe { Some(mem::transmute(wparam as u8 - 0x21)) },
		0x41..=0x5a => unsafe { Some(std::mem::transmute(wparam as u8 - 0x28)) },
		0x70..=0x83 => unsafe { Some(std::mem::transmute(wparam as u8 - 0x3d)) },
		_ => match VIRTUAL_KEY(wparam as u16) {
			VK_DOWN => Some(Key::ArrowDown),
			VK_LEFT => Some(Key::ArrowLeft),
			VK_RIGHT => Some(Key::ArrowRight),
			VK_UP => Some(Key::ArrowUp),
			VK_ESCAPE => Some(Key::Escape),
			VK_TAB => Some(Key::Tab),
			VK_BACK => Some(Key::Backspace),
			VK_RETURN => Some(Key::Enter),
			VK_SPACE => Some(Key::Space),
			VK_INSERT => Some(Key::Insert),
			VK_DELETE => Some(Key::Delete),
			VK_HOME => Some(Key::Home),
			VK_END => Some(Key::End),
			VK_PRIOR => Some(Key::PageUp),
			VK_NEXT => Some(Key::PageDown),
			_ => None,
		},
	}
}

fn get_clipboard_text() -> Option<String> {
	WindowsClipboardContext.get_contents().ok()
}
