use crate::input::{InputDevice, RawInput, RawInputEventDispatcher};
use std::collections::HashMap;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

pub struct Keyboard {
    inputs: Vec<RawInput>,
    input_names: HashMap<String, usize>,
    window_event_queue: Vec<KeyboardInput>,
}

impl Keyboard {
    pub fn new() -> Self {
        let inputs = vec![
            RawInput::new("1"),
            RawInput::new("2"),
            RawInput::new("3"),
            RawInput::new("4"),
            RawInput::new("5"),
            RawInput::new("6"),
            RawInput::new("7"),
            RawInput::new("8"),
            RawInput::new("9"),
            RawInput::new("0"),
            RawInput::new("a"),
            RawInput::new("b"),
            RawInput::new("c"),
            RawInput::new("d"),
            RawInput::new("e"),
            RawInput::new("f"),
            RawInput::new("g"),
            RawInput::new("h"),
            RawInput::new("i"),
            RawInput::new("j"),
            RawInput::new("k"),
            RawInput::new("l"),
            RawInput::new("m"),
            RawInput::new("n"),
            RawInput::new("o"),
            RawInput::new("p"),
            RawInput::new("q"),
            RawInput::new("r"),
            RawInput::new("s"),
            RawInput::new("t"),
            RawInput::new("u"),
            RawInput::new("v"),
            RawInput::new("w"),
            RawInput::new("x"),
            RawInput::new("y"),
            RawInput::new("z"),
            RawInput::new("escape"),
            RawInput::new("f1"),
            RawInput::new("f2"),
            RawInput::new("f3"),
            RawInput::new("f4"),
            RawInput::new("f5"),
            RawInput::new("f6"),
            RawInput::new("f7"),
            RawInput::new("f8"),
            RawInput::new("f9"),
            RawInput::new("f10"),
            RawInput::new("f11"),
            RawInput::new("f12"),
            RawInput::new("f13"),
            RawInput::new("f14"),
            RawInput::new("f15"),
            RawInput::new("f16"),
            RawInput::new("f17"),
            RawInput::new("f18"),
            RawInput::new("f19"),
            RawInput::new("f20"),
            RawInput::new("f21"),
            RawInput::new("f22"),
            RawInput::new("f23"),
            RawInput::new("f24"),
            RawInput::new("printscreen"),
            RawInput::new("scrolllock"),
            RawInput::new("pause"),
            RawInput::new("insert"),
            RawInput::new("home"),
            RawInput::new("delete"),
            RawInput::new("end"),
            RawInput::new("pagedown"),
            RawInput::new("pageup"),
            RawInput::new("left"),
            RawInput::new("up"),
            RawInput::new("right"),
            RawInput::new("down"),
            RawInput::new("backspace"),
            RawInput::new("enter"),
            RawInput::new("space"),
            RawInput::new("numlock"),
            RawInput::new("numpad0"),
            RawInput::new("numpad1"),
            RawInput::new("numpad2"),
            RawInput::new("numpad3"),
            RawInput::new("numpad4"),
            RawInput::new("numpad5"),
            RawInput::new("numpad6"),
            RawInput::new("numpad7"),
            RawInput::new("numpad8"),
            RawInput::new("numpad9"),
            RawInput::new("numpadadd"),
            RawInput::new("numpaddivide"),
            RawInput::new("numpaddecimal"),
            RawInput::new("numpadcomma"),
            RawInput::new("numpadenter"),
            RawInput::new("numpadequal"),
            RawInput::new("numpadmultiply"),
            RawInput::new("numpadsubtract"),
            RawInput::new("asterisk"),
            RawInput::new("at"),
            RawInput::new("backslash"),
            RawInput::new("colon"),
            RawInput::new("comma"),
            RawInput::new("equal"),
            RawInput::new("grave"),
            RawInput::new("lalt"),
            RawInput::new("lbracket"),
            RawInput::new("lcontrol"),
            RawInput::new("lshift"),
            RawInput::new("os"),
            RawInput::new("minus"),
            RawInput::new("plus"),
            RawInput::new("ralt"),
            RawInput::new("rbracket"),
            RawInput::new("rcontrol"),
            RawInput::new("rshift"),
            RawInput::new("os"),
            RawInput::new("semicolon"),
            RawInput::new("slash"),
            RawInput::new("tab"),
        ];
        let input_names = inputs
            .iter()
            .enumerate()
            .map(|(i, input)| (input.name.clone(), i))
            .collect();

        Self {
            inputs,
            input_names,
            window_event_queue: Vec::new(),
        }
    }

    pub fn handle_window_event(&mut self, event: KeyboardInput) {
        self.window_event_queue.push(event);
    }
}

impl InputDevice for Keyboard {
    fn name(&self) -> &str {
        "keyboard"
    }

    fn inputs(&self) -> &[RawInput] {
        &self.inputs
    }

    fn input(&self, name: &str) -> Option<&RawInput> {
        self.input_names.get(name).map(|&index| &self.inputs[index])
    }

    fn poll(&mut self, dispatcher: &mut RawInputEventDispatcher) {
        for event in self.window_event_queue.drain(..) {
            let name = if let Some(name) = event
                .virtual_keycode
                .and_then(virtual_keycode_into_raw_input_name)
            {
                name
            } else {
                continue;
            };

            let index = if let Some(&index) = self.input_names.get(name) {
                index
            } else {
                continue;
            };

            let input = &mut self.inputs[index];
            input.value = if event.state == ElementState::Pressed {
                1.0
            } else {
                0.0
            };

            dispatcher.dispatch(input);
        }
    }
}

fn virtual_keycode_into_raw_input_name(virtual_keycode: VirtualKeyCode) -> Option<&'static str> {
    match virtual_keycode {
        VirtualKeyCode::Key1 => Some("1"),
        VirtualKeyCode::Key2 => Some("2"),
        VirtualKeyCode::Key3 => Some("3"),
        VirtualKeyCode::Key4 => Some("4"),
        VirtualKeyCode::Key5 => Some("5"),
        VirtualKeyCode::Key6 => Some("6"),
        VirtualKeyCode::Key7 => Some("7"),
        VirtualKeyCode::Key8 => Some("8"),
        VirtualKeyCode::Key9 => Some("9"),
        VirtualKeyCode::Key0 => Some("0"),
        VirtualKeyCode::A => Some("a"),
        VirtualKeyCode::B => Some("b"),
        VirtualKeyCode::C => Some("c"),
        VirtualKeyCode::D => Some("d"),
        VirtualKeyCode::E => Some("e"),
        VirtualKeyCode::F => Some("f"),
        VirtualKeyCode::G => Some("g"),
        VirtualKeyCode::H => Some("h"),
        VirtualKeyCode::I => Some("i"),
        VirtualKeyCode::J => Some("j"),
        VirtualKeyCode::K => Some("k"),
        VirtualKeyCode::L => Some("l"),
        VirtualKeyCode::M => Some("m"),
        VirtualKeyCode::N => Some("n"),
        VirtualKeyCode::O => Some("o"),
        VirtualKeyCode::P => Some("p"),
        VirtualKeyCode::Q => Some("q"),
        VirtualKeyCode::R => Some("r"),
        VirtualKeyCode::S => Some("s"),
        VirtualKeyCode::T => Some("t"),
        VirtualKeyCode::U => Some("u"),
        VirtualKeyCode::V => Some("v"),
        VirtualKeyCode::W => Some("w"),
        VirtualKeyCode::X => Some("x"),
        VirtualKeyCode::Y => Some("y"),
        VirtualKeyCode::Z => Some("z"),
        VirtualKeyCode::Escape => Some("escape"),
        VirtualKeyCode::F1 => Some("f1"),
        VirtualKeyCode::F2 => Some("f2"),
        VirtualKeyCode::F3 => Some("f3"),
        VirtualKeyCode::F4 => Some("f4"),
        VirtualKeyCode::F5 => Some("f5"),
        VirtualKeyCode::F6 => Some("f6"),
        VirtualKeyCode::F7 => Some("f7"),
        VirtualKeyCode::F8 => Some("f8"),
        VirtualKeyCode::F9 => Some("f9"),
        VirtualKeyCode::F10 => Some("f10"),
        VirtualKeyCode::F11 => Some("f11"),
        VirtualKeyCode::F12 => Some("f12"),
        VirtualKeyCode::F13 => Some("f13"),
        VirtualKeyCode::F14 => Some("f14"),
        VirtualKeyCode::F15 => Some("f15"),
        VirtualKeyCode::F16 => Some("f16"),
        VirtualKeyCode::F17 => Some("f17"),
        VirtualKeyCode::F18 => Some("f18"),
        VirtualKeyCode::F19 => Some("f19"),
        VirtualKeyCode::F20 => Some("f20"),
        VirtualKeyCode::F21 => Some("f21"),
        VirtualKeyCode::F22 => Some("f22"),
        VirtualKeyCode::F23 => Some("f23"),
        VirtualKeyCode::F24 => Some("f24"),
        VirtualKeyCode::Snapshot => Some("printscreen"),
        VirtualKeyCode::Scroll => Some("scrolllock"),
        VirtualKeyCode::Pause => Some("pause"),
        VirtualKeyCode::Insert => Some("insert"),
        VirtualKeyCode::Home => Some("home"),
        VirtualKeyCode::Delete => Some("delete"),
        VirtualKeyCode::End => Some("end"),
        VirtualKeyCode::PageDown => Some("pagedown"),
        VirtualKeyCode::PageUp => Some("pageup"),
        VirtualKeyCode::Left => Some("left"),
        VirtualKeyCode::Up => Some("up"),
        VirtualKeyCode::Right => Some("right"),
        VirtualKeyCode::Down => Some("down"),
        VirtualKeyCode::Back => Some("backspace"),
        VirtualKeyCode::Return => Some("enter"),
        VirtualKeyCode::Space => Some("space"),
        VirtualKeyCode::Compose => None,
        VirtualKeyCode::Caret => None,
        VirtualKeyCode::Numlock => Some("numlock"),
        VirtualKeyCode::Numpad0 => Some("numpad0"),
        VirtualKeyCode::Numpad1 => Some("numpad1"),
        VirtualKeyCode::Numpad2 => Some("numpad2"),
        VirtualKeyCode::Numpad3 => Some("numpad3"),
        VirtualKeyCode::Numpad4 => Some("numpad4"),
        VirtualKeyCode::Numpad5 => Some("numpad5"),
        VirtualKeyCode::Numpad6 => Some("numpad6"),
        VirtualKeyCode::Numpad7 => Some("numpad7"),
        VirtualKeyCode::Numpad8 => Some("numpad8"),
        VirtualKeyCode::Numpad9 => Some("numpad9"),
        VirtualKeyCode::NumpadAdd => Some("numpadadd"),
        VirtualKeyCode::NumpadDivide => Some("numpaddivide"),
        VirtualKeyCode::NumpadDecimal => Some("numpaddecimal"),
        VirtualKeyCode::NumpadComma => Some("numpadcomma"),
        VirtualKeyCode::NumpadEnter => Some("numpadenter"),
        VirtualKeyCode::NumpadEquals => Some("numpadequal"),
        VirtualKeyCode::NumpadMultiply => Some("numpadmultiply"),
        VirtualKeyCode::NumpadSubtract => Some("numpadsubtract"),
        VirtualKeyCode::AbntC1 => None,
        VirtualKeyCode::AbntC2 => None,
        VirtualKeyCode::Apostrophe => None,
        VirtualKeyCode::Apps => None,
        VirtualKeyCode::Asterisk => Some("asterisk"),
        VirtualKeyCode::At => Some("at"),
        VirtualKeyCode::Ax => None,
        VirtualKeyCode::Backslash => Some("backslash"),
        VirtualKeyCode::Calculator => None,
        VirtualKeyCode::Capital => None,
        VirtualKeyCode::Colon => Some("colon"),
        VirtualKeyCode::Comma => Some("comma"),
        VirtualKeyCode::Convert => None,
        VirtualKeyCode::Equals => Some("equal"),
        VirtualKeyCode::Grave => Some("grave"),
        VirtualKeyCode::Kana => None,
        VirtualKeyCode::Kanji => None,
        VirtualKeyCode::LAlt => Some("lalt"),
        VirtualKeyCode::LBracket => Some("lbracket"),
        VirtualKeyCode::LControl => Some("lcontrol"),
        VirtualKeyCode::LShift => Some("lshift"),
        VirtualKeyCode::LWin => Some("os"),
        VirtualKeyCode::Mail => None,
        VirtualKeyCode::MediaSelect => None,
        VirtualKeyCode::MediaStop => None,
        VirtualKeyCode::Minus => Some("minus"),
        VirtualKeyCode::Mute => None,
        VirtualKeyCode::MyComputer => None,
        VirtualKeyCode::NavigateForward => None,
        VirtualKeyCode::NavigateBackward => None,
        VirtualKeyCode::NextTrack => None,
        VirtualKeyCode::NoConvert => None,
        VirtualKeyCode::OEM102 => None,
        VirtualKeyCode::Period => None,
        VirtualKeyCode::PlayPause => None,
        VirtualKeyCode::Plus => Some("plus"),
        VirtualKeyCode::Power => None,
        VirtualKeyCode::PrevTrack => None,
        VirtualKeyCode::RAlt => Some("ralt"),
        VirtualKeyCode::RBracket => Some("rbracket"),
        VirtualKeyCode::RControl => Some("rcontrol"),
        VirtualKeyCode::RShift => Some("rshift"),
        VirtualKeyCode::RWin => Some("os"),
        VirtualKeyCode::Semicolon => Some("semicolon"),
        VirtualKeyCode::Slash => Some("slash"),
        VirtualKeyCode::Sleep => None,
        VirtualKeyCode::Stop => None,
        VirtualKeyCode::Sysrq => None,
        VirtualKeyCode::Tab => Some("tab"),
        VirtualKeyCode::Underline => None,
        VirtualKeyCode::Unlabeled => None,
        VirtualKeyCode::VolumeDown => None,
        VirtualKeyCode::VolumeUp => None,
        VirtualKeyCode::Wake => None,
        VirtualKeyCode::WebBack => None,
        VirtualKeyCode::WebFavorites => None,
        VirtualKeyCode::WebForward => None,
        VirtualKeyCode::WebHome => None,
        VirtualKeyCode::WebRefresh => None,
        VirtualKeyCode::WebSearch => None,
        VirtualKeyCode::WebStop => None,
        VirtualKeyCode::Yen => None,
        VirtualKeyCode::Copy => None,
        VirtualKeyCode::Paste => None,
        VirtualKeyCode::Cut => None,
    }
}
