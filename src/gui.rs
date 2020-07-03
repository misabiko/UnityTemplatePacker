extern crate iced;

use iced::{button, Button, Text, Column, Sandbox, Element, text_input, Container, Length, Settings};

#[derive(Default)]
pub struct TemplatePacker {
	src_project_input: text_input::State,
	src_project_value: String,
	editor_input: text_input::State,
	editor_value: String,
	pack_button: button::State,
}

#[derive(Debug, Clone)]
pub enum Message {
	Pack,
	SrcProjectChanged(String),
	EditorChanged(String),
}

impl Sandbox for TemplatePacker {
	type Message = Message;

	fn new() -> Self {
		Self::default()
	}

	fn title(&self) -> String {
		String::from("Unity Template Packer")
	}

	fn update(&mut self, message: Message) {
		match message {
			Message::Pack => {
				println!("Pack!");
				//pack(Path::new(&self.src_project_value), Path::new(&self.editor_value)).unwrap();
			}
			Message::SrcProjectChanged(value) => self.src_project_value = value,
			Message::EditorChanged(value) => self.editor_value = value,
		}
	}

	fn view(&mut self) -> Element<Message> {
		let column = Column::new()
			.spacing(20)
			.padding(20)
			.max_width(600)
			.push(Text::new("Source Project Path:"))
			.push(
				text_input::TextInput::new(
					&mut self.src_project_input,
					"Source Project Path",
					&self.src_project_value,
					Message::SrcProjectChanged,
				)
					.padding(10)
					.size(20)
			)
			.push(Text::new("Editor Project Path:"))
			.push(
				text_input::TextInput::new(
					&mut self.editor_input,
					"Editor Path",
					&self.editor_value,
					Message::EditorChanged,
				)
					.padding(10)
					.size(20)
			)
			.push(
				Button::new(&mut self.pack_button, Text::new("Pack"))
					.on_press(Message::Pack)
			);

		Container::new(column)
			.width(Length::Fill)
			.height(Length::Fill)
			.center_x()
			.into()
	}
}

impl TemplatePacker {
	pub fn run_default() {
		TemplatePacker::run(Settings::default());
	}
}