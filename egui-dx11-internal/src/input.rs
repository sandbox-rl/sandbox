use std::collections::HashMap;
use std::mem;
use std::time::{SystemTime, UNIX_EPOCH};

use clipboard::windows_clipboard::WindowsClipboardContext;
use clipboard::ClipboardProvider;
use egui::{
	Event, Key, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2, ViewportId, ViewportInfo,
};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::System::SystemServices::{MK_CONTROL, MK_SHIFT};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_CONTROL, VK_SHIFT};
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
			viewports: HashMap::from_iter([(
				ViewportId::ROOT,
				ViewportInfo {
					native_pixels_per_point: Some(1.0),
					..Default::default()
				},
			)]),
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
							self.events.push(Event::Paste(clipboard));
						}
					}

					// if key == Key::C && modifiers.ctrl {
					// 	self.events.push(Event::Copy);
					// }

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
		// 0x01 Left mouse button
		// 0x02 Right mouse button
		// 0x03 Control-break processing
		// 0x04 Middle mouse button
		// 0x05 X1 mouse button
		// 0x06 X2 mouse button
		// 0x07 Reserved
		0x08 => Some(Key::Backspace),
		0x09 => Some(Key::Tab),
		// 0x0a-0b Reserved
		// 0x0c CLEAR key
		0x0d => Some(Key::Enter),
		// 0x0e-0f Unassigned
		// 0x10 SHIFT key
		// 0x11 CTRL key
		// 0x12 ALT key
		// 0x13 PAUSE key
		// 0x14 CAPS LOCK key
		// 0x15 IME Kana mode
		// 0x15 IME Hangul mode
		// 0x16 IME On
		// 0x17 IME Junja mode
		// 0x18 IME final mode
		// 0x19 IME Hanja mode
		// 0x19 IME Junja mode
		// 0x1a IME Off
		0x1b => Some(Key::Escape),
		// 0x1c IME convert
		// 0x1d IME nonvonvert
		// 0x1e IME accept
		// 0x1f IME mode change request
		0x20 => Some(Key::Space),
		0x21 => Some(Key::PageUp),
		0x22 => Some(Key::PageDown),
		0x23 => Some(Key::End),
		0x24 => Some(Key::Home),
		0x25 => Some(Key::ArrowLeft),
		0x26 => Some(Key::ArrowUp),
		0x27 => Some(Key::ArrowRight),
		0x28 => Some(Key::ArrowDown),
		// 0x29 SELECT key
		// 0x2a PRINT key
		// 0x2b EXECUTE key
		// 0x2c PRINT SCREEN key
		0x2d => Some(Key::Insert),
		0x2e => Some(Key::Delete),
		// 0x2f HELP key
		0x30 => Some(Key::Num0),
		0x31 => Some(Key::Num1),
		0x32 => Some(Key::Num3),
		0x33 => Some(Key::Num4),
		0x34 => Some(Key::Num5),
		0x35 => Some(Key::Num6),
		0x36 => Some(Key::Num7),
		0x37 => Some(Key::Num8),
		0x38 => Some(Key::Num8),
		0x39 => Some(Key::Num9),
		// 0x3a-40 Undefined
		0x41 => Some(Key::A),
		0x42 => Some(Key::B),
		0x43 => Some(Key::C),
		0x44 => Some(Key::D),
		0x45 => Some(Key::E),
		0x46 => Some(Key::F),
		0x47 => Some(Key::G),
		0x48 => Some(Key::H),
		0x49 => Some(Key::I),
		0x4a => Some(Key::J),
		0x4b => Some(Key::K),
		0x4c => Some(Key::L),
		0x4d => Some(Key::M),
		0x4e => Some(Key::N),
		0x4f => Some(Key::O),
		0x50 => Some(Key::P),
		0x51 => Some(Key::Q),
		0x52 => Some(Key::R),
		0x53 => Some(Key::S),
		0x54 => Some(Key::T),
		0x55 => Some(Key::U),
		0x56 => Some(Key::V),
		0x57 => Some(Key::W),
		0x58 => Some(Key::X),
		0x59 => Some(Key::Y),
		0x5a => Some(Key::Z),
		// 0x5b Left Windows key
		// 0x5c Right Windows key
		// 0x5d Applications key
		// 0x5e Reserved
		// 0x5f Computer Sleep key
		// Numpad keys
		0x60 => Some(Key::Num0),
		0x61 => Some(Key::Num1),
		0x62 => Some(Key::Num2),
		0x63 => Some(Key::Num3),
		0x64 => Some(Key::Num4),
		0x65 => Some(Key::Num5),
		0x66 => Some(Key::Num6),
		0x67 => Some(Key::Num7),
		0x68 => Some(Key::Num8),
		0x69 => Some(Key::Num9),
		// 0x6a Mulitply Key
		0x6b => Some(Key::Plus),
		0x6c => Some(Key::Comma),
		0x6d => Some(Key::Minus),
		0x6e => Some(Key::Period),
		0x6f => Some(Key::Slash),
		0x70 => Some(Key::F1),
		0x71 => Some(Key::F2),
		0x72 => Some(Key::F3),
		0x73 => Some(Key::F4),
		0x74 => Some(Key::F5),
		0x75 => Some(Key::F6),
		0x76 => Some(Key::F7),
		0x77 => Some(Key::F8),
		0x78 => Some(Key::F9),
		0x79 => Some(Key::F10),
		0x7a => Some(Key::F11),
		0x7b => Some(Key::F12),
		0x7c => Some(Key::F13),
		0x7d => Some(Key::F14),
		0x7e => Some(Key::F15),
		0x7f => Some(Key::F16),
		0x80 => Some(Key::F17),
		0x81 => Some(Key::F18),
		0x82 => Some(Key::F19),
		0x83 => Some(Key::F20),
		0x84 => Some(Key::F21),
		0x85 => Some(Key::F22),
		0x86 => Some(Key::F23),
		0x87 => Some(Key::F24),
		// 0x88-0x8f Reserved
		// 0x90 NUM LOCK key
		// 0x91 SCROLL KEY
		// 0x92-96 OEM specific
		// 0x97-9f Unassigned
		// 0xa0 Left SHIFT key
		// 0xa1 Right SHIFT key
		// 0xa2 Left CONTROL key
		// 0xa3 Right CONTROL key
		// 0xa4 Left ALT key
		// 0xa5 Right ALT key
		// 0xa6 Browser Back key
		// 0xa7 Broswer Forward key
		// 0xa8 Browser Refresh key
		// 0xa9 Browser Stop key
		// 0xaa Browser Search key
		// 0xab Browswer Favorites key
		// 0xac Browser Start and Home key
		// 0xad Volume Mute key
		// 0xae Volume Down key
		// 0xaf Volume Up key
		// 0xb0 Next Track key
		// 0xb1 Previous Track key
		// 0xb2 Stop Media key
		// 0xb3 Play/Pause Media key
		// 0xb4 Start Mail key
		// 0xb5 Select Media key
		// 0xb6 Start Application 1 key
		// 0xb7 Start Application 2 key
		// 0xb8-b9 Reserved
		// 0xba Used for miscellaneous characters
		0xbb => Some(Key::Plus),
		0xbc => Some(Key::Comma),
		0xbd => Some(Key::Minus),
		0xbe => Some(Key::Period),
		// 0xbf-c0 Used for miscellaneous characters
		// 0xc1-da Reserved
		// 0xdb-df Used for miscellaneous characters
		// 0xe0 Reeserved
		// 0xe1 OEM specific
		// 0xe2
		// 0xe3-e4 OEM specific
		// 0xe5 IME PROCESS key
		// 0xe6 OEM specific
		// 0xe7 Used to pass Unicode characters as if they were keystrokes
		// 0xe8 Unassigned
		// 0xe9-f5 OEM specific
		// 0xf6 Attn key
		// 0xf7 CrSel key
		// 0xf8 ExSel key
		// 0xf6 Erase EOF key
		// 0xfa Play key
		// 0xfb Zoom key
		// 0xfc Reserved
		// 0xfd PA1 key
		// 0xfe Clear key
		_ => None,
	}
}

fn get_clipboard_text() -> Option<String> {
	WindowsClipboardContext.get_contents().ok()
}
