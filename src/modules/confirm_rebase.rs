use crate::{
	application::AppData,
	components::confirm::{Confirm, Confirmed, INPUT_OPTIONS},
	input::{Event, InputOptions, KeyBindings},
	module::{ExitStatus, Module, State},
	process::Results,
	view::{RenderContext, ViewData},
};

pub(crate) struct ConfirmRebase {
	dialog: Confirm,
}

impl Module for ConfirmRebase {
	fn build_view_data(&mut self, _: &RenderContext) -> &ViewData {
		self.dialog.get_view_data()
	}

	fn input_options(&self) -> &InputOptions {
		&INPUT_OPTIONS
	}

	fn read_event(&self, event: Event, key_bindings: &KeyBindings) -> Event {
		Confirm::read_event(event, key_bindings)
	}

	fn handle_event(&mut self, event: Event) -> Results {
		let confirmed = self.dialog.handle_event(event);
		let mut results = Results::new();
		match confirmed {
			Confirmed::Yes => {
				results.exit_status(ExitStatus::Good);
			},
			Confirmed::No => {
				results.state(State::List);
			},
			Confirmed::Other => {},
		}
		results
	}
}

impl ConfirmRebase {
	pub(crate) fn new(app_data: &AppData) -> Self {
		let config = app_data.config();
		Self {
			dialog: Confirm::new(
				"Are you sure you want to rebase",
				&config.key_bindings.confirm_yes,
				&config.key_bindings.confirm_no,
			),
		}
	}
}
#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		assert_rendered_output,
		assert_results,
		input::{KeyCode, StandardEvent},
		process::Artifact,
		test_helpers::{assertions::assert_rendered_output::AssertRenderOptions, testers},
	};

	#[test]
	fn build_view_data() {
		testers::module(&["pick aaa comment"], &[], None, |test_context| {
			let mut module = ConfirmRebase::new(&test_context.app_data());
			let view_data = test_context.build_view_data(&mut module);
			assert_rendered_output!(
			Options AssertRenderOptions::INCLUDE_TRAILING_WHITESPACE | AssertRenderOptions::INCLUDE_STYLE,
			view_data,
			"{TITLE}",
			"{BODY}",
			"{Normal}Are you sure you want to rebase (y/n)? "
			);
		});
	}

	#[test]
	fn handle_event_yes() {
		testers::module(
			&["pick aaa comment"],
			&[Event::from(StandardEvent::Yes)],
			None,
			|mut test_context| {
				let mut module = ConfirmRebase::new(&test_context.app_data());
				assert_results!(
					test_context.handle_event(&mut module),
					Artifact::Event(Event::from(StandardEvent::Yes)),
					Artifact::ExitStatus(ExitStatus::Good)
				);
			},
		);
	}

	#[test]
	fn handle_event_no() {
		testers::module(
			&["pick aaa comment"],
			&[Event::from(StandardEvent::No)],
			None,
			|mut test_context| {
				let mut module = ConfirmRebase::new(&test_context.app_data());
				assert_results!(
					test_context.handle_event(&mut module),
					Artifact::Event(Event::from(StandardEvent::No)),
					Artifact::ChangeState(State::List)
				);
			},
		);
	}

	#[test]
	fn handle_event_no_match_key() {
		testers::module(
			&["pick aaa comment"],
			&[Event::from(KeyCode::Null)],
			None,
			|mut test_context| {
				let mut module = ConfirmRebase::new(&test_context.app_data());
				assert_results!(
					test_context.handle_event(&mut module),
					Artifact::Event(Event::from(KeyCode::Null))
				);
			},
		);
	}
}
