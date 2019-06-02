use crate::action::Action;
use crate::git_interactive::GitInteractive;

use crate::config::Config;
use crate::constants::{LIST_HELP_LINES, VISUAL_MODE_HELP_LINES};
use crate::exit_status::ExitStatus;
use crate::input::{Input, InputHandler};
use crate::process::{ProcessResult, ProcessResultBuilder, State};
use crate::view::View;
use crate::window::Window;
use core::borrow::Borrow;
use std::cell::RefCell;
use std::process::Command;
use std::process::ExitStatus as ProcessExitStatus;
use unicode_segmentation::UnicodeSegmentation;

pub struct Application<'a> {
	config: &'a Config,
	edit_content: String,
	edit_content_cursor: usize,
	exit_status: Option<ExitStatus>,
	git_interactive: GitInteractive,
	input_handler: &'a InputHandler<'a>,
	state: RefCell<State>,
	view: View<'a>,
}

impl<'a> Application<'a> {
	pub fn new(
		git_interactive: GitInteractive,
		view: View<'a>,
		input_handler: &'a InputHandler<'a>,
		config: &'a Config,
	) -> Self
	{
		Self {
			config,
			edit_content: String::from(""),
			edit_content_cursor: 0,
			exit_status: None,
			git_interactive,
			input_handler,
			state: RefCell::new(State::List),
			view,
		}
	}

	pub fn run(&mut self) -> Result<Option<ExitStatus>, String> {
		self.handle_resize();
		while self.exit_status.is_none() {
			// process based on input, allowed to change state
			self.process();
			// draw output for state, including state change from process
			self.draw();
			// handle input for state
			self.handle_input();
		}
		self.exit_end()?;
		Ok(self.exit_status)
	}

	fn get_cursor_index(&self) -> usize {
		*self.git_interactive.get_selected_line_index() - 1
	}

	fn process(&mut self) {
		let process_result = match self.get_state() {
			State::ConfirmAbort => ProcessResult::new(),
			State::ConfirmRebase => ProcessResult::new(),
			State::Edit => ProcessResult::new(),
			State::EditFinish => self.process_edit_finish(),
			State::Error { .. } => ProcessResult::new(),
			State::Exiting => ProcessResult::new(),
			State::ExternalEditor(return_state) => self.process_external_editor(return_state.borrow()),
			State::ExternalEditorError => self.process_external_editor_error(),
			State::ExternalEditorFinish(_) => self.process_external_editor_finish(),
			State::Help(_) => ProcessResult::new(),
			State::List => self.process_list(),
			State::ShowCommit => self.process_show_commit(),
			State::VisualMode => self.process_list(),
			State::WindowSizeError(_) => ProcessResult::new(),
		};

		if let Some(exit_status) = process_result.exit_status {
			self.exit_status = Some(exit_status);
		}

		if let Some(new_state) = process_result.state {
			self.set_state(new_state);
		}
	}

	fn process_edit_finish(&mut self) -> ProcessResult {
		self.git_interactive.edit_selected_line(self.edit_content.as_str());
		ProcessResultBuilder::new().state(State::List).build()
	}

	fn process_external_editor(&mut self, return_state: &State) -> ProcessResult {
		let mut result = ProcessResultBuilder::new();

		result = if let Err(e) = self.run_editor() {
			result.error(e.as_str(), State::ExternalEditorError)
		}
		else {
			result.state(State::ExternalEditorFinish(Box::new(return_state.clone())))
		};

		result.build()
	}

	fn process_external_editor_finish(&mut self) -> ProcessResult {
		let mut result = ProcessResultBuilder::new();
		result = if let Err(e) = self.git_interactive.reload_file(self.config.comment_char.as_str()) {
			result.error(e.as_str(), State::List)
		}
		else if self.git_interactive.get_lines().is_empty() {
			result.error("Rebase empty", State::List)
		}
		else {
			result.state(State::List)
		};

		result.build()
	}

	fn process_external_editor_error(&mut self) -> ProcessResult {
		ProcessResultBuilder::new()
			.state(State::Exiting)
			.exit_status(
				if self.git_interactive.get_lines().is_empty() {
					ExitStatus::Good
				}
				else {
					ExitStatus::StateError
				},
			)
			.build()
	}

	fn process_list(&mut self) -> ProcessResult {
		let lines = self.git_interactive.get_lines();
		let selected_index = self.get_cursor_index();
		self.view.update_main_top(lines.len(), selected_index);
		ProcessResult::new()
	}

	fn process_show_commit(&mut self) -> ProcessResult {
		let mut result = ProcessResultBuilder::new();
		if let Err(e) = self.git_interactive.load_commit_stats() {
			result = result.error(e.as_str(), State::List);
		}
		result.build()
	}

	fn draw(&self) {
		self.view.clear();
		match self.get_state() {
			State::ConfirmAbort => self.draw_confirm_abort(),
			State::ConfirmRebase => self.draw_confirm_rebase(),
			State::Edit => self.draw_edit(),
			State::EditFinish => {},
			State::Error { message, .. } => self.draw_error(message.as_str()),
			State::Exiting => self.draw_exiting(),
			State::ExternalEditor(_) => {},
			State::ExternalEditorError => {},
			State::ExternalEditorFinish(_) => {},
			State::Help(help_state) => self.draw_help(help_state.borrow()),
			State::List => self.draw_main(false),
			State::VisualMode => self.draw_main(false),
			State::ShowCommit => self.draw_show_commit(),
			State::WindowSizeError(_) => self.draw_window_size_error(),
		}
		self.view.refresh();
	}

	fn draw_confirm_abort(&self) {
		self.view.draw_confirm("Are you sure you want to abort");
	}

	fn draw_confirm_rebase(&self) {
		self.view.draw_confirm("Are you sure you want to rebase");
	}

	fn draw_error(&self, error_message: &str) {
		self.view.draw_error(error_message);
	}

	fn draw_show_commit(&self) {
		self.view.draw_show_commit(self.git_interactive.get_commit_stats());
	}

	fn draw_main(&self, visual_mode: bool) {
		self.view.draw_main(
			self.git_interactive.get_lines(),
			self.get_cursor_index(),
			if visual_mode {
				Some(self.git_interactive.get_visual_start_index() - 1)
			}
			else {
				None
			},
		);
	}

	fn draw_edit(&self) {
		self.view
			.draw_edit(self.edit_content.as_str(), self.edit_content_cursor);
	}

	fn draw_help(&self, help_state: &State) {
		self.view.draw_help(
			if *help_state == State::List {
				LIST_HELP_LINES
			}
			else {
				VISUAL_MODE_HELP_LINES
			},
		);
	}

	fn draw_exiting(&self) {
		self.view.draw_exiting();
	}

	fn draw_window_size_error(&self) {
		self.view.draw_window_size_error();
	}

	fn handle_resize(&self) {
		let check = self.view.check_window_size();
		let state = self.get_state();
		if let State::WindowSizeError(return_state) = state {
			if check {
				self.set_state(*return_state);
			}
		}
		else if !check {
			self.set_state(State::WindowSizeError(Box::new(self.get_state())));
		}
	}

	fn get_input(&self) -> Input {
		let input = self.input_handler.get_input();
		if let Input::Resize = input {
			self.handle_resize();
		}
		input
	}

	fn get_confirm(&mut self) -> Input {
		let input = self.input_handler.get_confirm();
		if let Input::Resize = input {
			self.handle_resize();
		}
		input
	}

	fn handle_input(&mut self) {
		if let Some(new_state) = match self.get_state() {
			State::ConfirmAbort => self.handle_confirm_abort_input(),
			State::ConfirmRebase => self.handle_confirm_rebase_input(),
			State::Edit => self.handle_edit(),
			State::EditFinish => None,
			State::Error { return_state, .. } => self.handle_error_input(return_state.borrow()),
			State::Exiting => None,
			State::ExternalEditor(return_state) => self.handle_external_editor_input(return_state.borrow()),
			State::ExternalEditorError => None,
			State::ExternalEditorFinish(_) => None,
			State::Help(help_state) => self.handle_help_input(help_state.borrow()),
			State::List => self.handle_list_input(),
			State::VisualMode => self.handle_visual_mode_input(),
			State::ShowCommit => self.handle_show_commit_input(),
			State::WindowSizeError(_) => self.handle_window_size_error_input(),
		} {
			self.set_state(new_state);
		}
	}

	fn handle_help_input(&mut self, help_state: &State) -> Option<State> {
		let help_lines = if *help_state == State::List {
			LIST_HELP_LINES
		}
		else {
			VISUAL_MODE_HELP_LINES
		};
		match self.get_input() {
			Input::MoveCursorDown => {
				self.view.update_help_top(false, false, help_lines);
			},
			Input::MoveCursorUp => {
				self.view.update_help_top(true, false, help_lines);
			},
			Input::Resize => {
				self.view.update_help_top(true, true, help_lines);
			},
			_ => {
				self.set_state(help_state.clone());
			},
		}
		None
	}

	fn handle_visual_mode_input(&mut self) -> Option<State> {
		match self.get_input() {
			Input::MoveCursorDown => {
				self.git_interactive.move_cursor_down(1);
			},
			Input::MoveCursorUp => {
				self.git_interactive.move_cursor_up(1);
			},
			Input::MoveCursorPageDown => {
				self.git_interactive.move_cursor_down(5);
			},
			Input::MoveCursorPageUp => {
				self.git_interactive.move_cursor_up(5);
			},
			Input::ActionDrop => self.git_interactive.set_visual_range_action(Action::Drop),
			Input::ActionEdit => self.git_interactive.set_visual_range_action(Action::Edit),
			Input::ActionFixup => self.git_interactive.set_visual_range_action(Action::Fixup),
			Input::ActionPick => self.git_interactive.set_visual_range_action(Action::Pick),
			Input::ActionReword => self.git_interactive.set_visual_range_action(Action::Reword),
			Input::ActionSquash => self.git_interactive.set_visual_range_action(Action::Squash),
			Input::SwapSelectedDown => self.git_interactive.swap_visual_range_down(),
			Input::SwapSelectedUp => self.git_interactive.swap_visual_range_up(),
			Input::ToggleVisualMode => {
				return Some(State::List);
			},
			Input::Help => {
				self.view.update_help_top(false, true, VISUAL_MODE_HELP_LINES);
				return Some(State::Help(Box::new(self.get_state())));
			},
			_ => {},
		}
		None
	}

	fn handle_show_commit_input(&mut self) -> Option<State> {
		match self.get_input() {
			Input::MoveCursorDown => {
				self.view
					.update_commit_top(false, false, self.git_interactive.get_commit_stats_length());
			},
			Input::MoveCursorUp => {
				self.view
					.update_commit_top(true, false, self.git_interactive.get_commit_stats_length());
			},
			Input::Resize => {
				self.view
					.update_commit_top(true, false, self.git_interactive.get_commit_stats_length());
			},
			_ => {
				return Some(State::List);
			},
		}
		None
	}

	fn handle_confirm_abort_input(&mut self) -> Option<State> {
		match self.get_confirm() {
			Input::Yes => {
				self.exit_abort();
				return Some(State::Exiting);
			},
			Input::No => {
				return Some(State::List);
			},
			_ => {},
		}
		None
	}

	fn handle_confirm_rebase_input(&mut self) -> Option<State> {
		match self.get_confirm() {
			Input::Yes => {
				self.exit_finish();
				return Some(State::Exiting);
			},
			Input::No => {
				return Some(State::List);
			},
			_ => {},
		}
		None
	}

	fn handle_edit(&mut self) -> Option<State> {
		loop {
			match self.input_handler.get_character() {
				Input::Character(c) => {
					let start = UnicodeSegmentation::graphemes(self.edit_content.as_str(), true)
						.take(self.edit_content_cursor)
						.collect::<String>();
					let end = UnicodeSegmentation::graphemes(self.edit_content.as_str(), true)
						.skip(self.edit_content_cursor)
						.collect::<String>();
					self.edit_content = format!("{}{}{}", start, c, end);
					self.edit_content_cursor += 1;
				},
				Input::Backspace => {
					if self.edit_content_cursor == 0 {
						break;
					}
					let start = UnicodeSegmentation::graphemes(self.edit_content.as_str(), true)
						.take(self.edit_content_cursor - 1)
						.collect::<String>();
					let end = UnicodeSegmentation::graphemes(self.edit_content.as_str(), true)
						.skip(self.edit_content_cursor)
						.collect::<String>();
					self.edit_content = format!("{}{}", start, end);
					self.edit_content_cursor -= 1;
				},
				Input::Delete => {
					let length = UnicodeSegmentation::graphemes(self.edit_content.as_str(), true).count();
					if self.edit_content_cursor == length {
						break;
					}
					let start = UnicodeSegmentation::graphemes(self.edit_content.as_str(), true)
						.take(self.edit_content_cursor)
						.collect::<String>();
					let end = UnicodeSegmentation::graphemes(self.edit_content.as_str(), true)
						.skip(self.edit_content_cursor + 1)
						.collect::<String>();
					self.edit_content = format!("{}{}", start, end);
				},
				Input::MoveCursorRight => {
					let length = UnicodeSegmentation::graphemes(self.edit_content.as_str(), true).count();
					if self.edit_content_cursor < length {
						self.edit_content_cursor += 1;
					}
				},
				Input::MoveCursorLeft => {
					if self.edit_content_cursor != 0 {
						self.edit_content_cursor -= 1;
					}
				},
				Input::Enter => return Some(State::EditFinish),
				_ => {
					continue;
				},
			}
			break;
		}
		None
	}

	fn handle_error_input(&mut self, return_state: &State) -> Option<State> {
		match self.get_input() {
			Input::Resize => {},
			_ => {
				return Some(return_state.clone());
			},
		}
		None
	}

	fn handle_external_editor_input(&mut self, return_state: &State) -> Option<State> {
		match self.get_input() {
			Input::Resize => {},
			_ => {
				return Some(return_state.clone());
			},
		}
		None
	}

	fn handle_list_input(&mut self) -> Option<State> {
		match self.get_input() {
			Input::Help => {
				self.view.update_help_top(false, true, LIST_HELP_LINES);
				return Some(State::Help(Box::new(self.get_state())));
			},
			Input::ShowCommit => {
				if !self.git_interactive.get_selected_line_hash().is_empty() {
					self.view.update_commit_top(false, true, 0);
					return Some(State::ShowCommit);
				}
			},
			Input::Abort => return Some(State::ConfirmAbort),
			Input::ForceAbort => {
				self.exit_abort();
				return Some(State::Exiting);
			},
			Input::Rebase => return Some(State::ConfirmRebase),
			Input::ForceRebase => {
				self.exit_finish();
				return Some(State::Exiting);
			},
			Input::ActionBreak => self.git_interactive.toggle_break(),
			Input::ActionDrop => self.set_selected_line_action(Action::Drop),
			Input::ActionEdit => self.set_selected_line_action(Action::Edit),
			Input::ActionFixup => self.set_selected_line_action(Action::Fixup),
			Input::ActionPick => self.set_selected_line_action(Action::Pick),
			Input::ActionReword => self.set_selected_line_action(Action::Reword),
			Input::ActionSquash => self.set_selected_line_action(Action::Squash),
			Input::Edit => {
				if *self.git_interactive.get_selected_line_action() == Action::Exec {
					self.edit_content = self.git_interactive.get_selected_line_edit_content().clone();
					self.edit_content_cursor = UnicodeSegmentation::graphemes(self.edit_content.as_str(), true).count();
					return Some(State::Edit);
				}
			},
			Input::SwapSelectedDown => self.git_interactive.swap_selected_down(),
			Input::SwapSelectedUp => self.git_interactive.swap_selected_up(),
			Input::MoveCursorDown => self.git_interactive.move_cursor_down(1),
			Input::MoveCursorUp => self.git_interactive.move_cursor_up(1),
			Input::MoveCursorPageDown => self.git_interactive.move_cursor_down(5),
			Input::MoveCursorPageUp => self.git_interactive.move_cursor_up(5),
			Input::ToggleVisualMode => {
				self.git_interactive.start_visual_mode();
				return Some(State::VisualMode);
			},
			Input::OpenInEditor => return Some(State::ExternalEditor(Box::new(self.get_state()))),
			_ => {},
		}
		None
	}

	fn handle_window_size_error_input(&mut self) -> Option<State> {
		self.get_input();
		None
	}

	fn run_editor(&mut self) -> Result<(), String> {
		self.git_interactive.write_file()?;
		let filepath = self.git_interactive.get_filepath();
		let callback = || -> Result<ProcessExitStatus, String> {
			// TODO: This doesn't handle editor with arguments (e.g. EDITOR="edit --arg")
			Command::new(&self.config.editor)
				.arg(filepath.as_os_str())
				.status()
				.map_err(|e| {
					format!(
						"Unable to run editor ({}):\n{}",
						self.config.editor.to_string_lossy(),
						e.to_string()
					)
				})
		};
		let exit_status: ProcessExitStatus = Window::leave_temporarily(callback)?;

		if !exit_status.success() {
			return Err(String::from("Editor returned non-zero exit status."));
		}

		Ok(())
	}

	fn set_selected_line_action(&mut self, action: Action) {
		self.git_interactive.set_selected_line_action(action);
		if self.config.auto_select_next {
			self.git_interactive.move_cursor_down(1);
		}
	}

	fn get_state(&self) -> State {
		self.state.borrow().clone()
	}

	fn set_state(&self, new_state: State) {
		self.state.replace(new_state);
	}

	fn exit_abort(&mut self) {
		self.git_interactive.clear();
		self.exit_finish();
	}

	fn exit_finish(&mut self) {
		self.exit_status = Some(ExitStatus::Good);
	}

	fn exit_end(&mut self) -> Result<(), String> {
		match self.git_interactive.write_file() {
			Ok(_) => {},
			Err(msg) => {
				self.exit_status = Some(ExitStatus::FileWriteError);
				return Err(msg);
			},
		}
		Ok(())
	}
}
