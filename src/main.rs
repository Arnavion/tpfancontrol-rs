#![deny(rust_2018_idioms, warnings)]

mod acpi;

mod error;
use error::Error;

mod model;

fn main() -> Result<(), Error> {
	let mut state = model::State::new(std::time::Duration::from_secs(5))?;

	let mut window = cursive::Cursive::termion();
	window.set_fps(2);

	window.add_fullscreen_layer(render(&state));

	loop {
		state.update_sensors();

		let visible_temp_sensors =
			window
			.call_on_id(VISIBLE_TEMP_SENSORS_GROUP_ID,
				|visible_temp_sensors_group: &mut RadioGroupView<model::VisibleTempSensors>| visible_temp_sensors_group.0.selection())
			.map_or_else(Default::default, |visible_temp_sensors| *visible_temp_sensors);

		let temp_scale =
			window
			.call_on_id(TEMP_SCALE_GROUP_ID,
				|temp_scale_group: &mut RadioGroupView<acpi::TempScale>| temp_scale_group.0.selection())
			.map_or_else(Default::default, |temp_scale| *temp_scale);

		let desired_fan_mode =
			window
			.call_on_id(FAN_SPEED_GROUP_ID,
				|fan_speed_group: &mut RadioGroupView<model::DesiredFanMode>| fan_speed_group.0.selection())
			.map_or_else(Default::default, |desired_fan_mode| *desired_fan_mode);

		let desired_manual_fan_level =
			window
			.call_on_id(DESIRED_MANUAL_FAN_LEVEL_ID,
				|desired_manual_fan_level: &mut cursive::views::SelectView<model::DesiredManualFanLevel>| desired_manual_fan_level.selection())
			.and_then(|desired_manual_fan_level| desired_manual_fan_level)
			.map_or_else(Default::default, |desired_manual_fan_level| *desired_manual_fan_level);

		state = model::State {
			visible_temp_sensors,
			temp_scale,

			desired_fan_mode,
			desired_manual_fan_level,

			..state
		};

		if state.fan_is_writable {
			let fan_level = match state.desired_fan_mode {
				model::DesiredFanMode::Bios => acpi::FanLevel::Auto,
				model::DesiredFanMode::Smart => {
					if let Ok(temps) = &state.temps {
						if let Some(Some(max_temp)) = temps.into_iter().max() {
							let mut computed_desired_manual_fan_level = model::DesiredManualFanLevel::FullSpeed;
							for (lower_bound, desired_manual_fan_level) in &state.config.fan_level {
								if max_temp > lower_bound {
									computed_desired_manual_fan_level = *desired_manual_fan_level;
								}
							}

							match computed_desired_manual_fan_level {
								model::DesiredManualFanLevel::Firmware(fan_firmware_level) => acpi::FanLevel::Firmware(fan_firmware_level),
								model::DesiredManualFanLevel::FullSpeed => acpi::FanLevel::FullSpeed,
							}
						}
						else {
							acpi::FanLevel::FullSpeed
						}
					}
					else {
						acpi::FanLevel::FullSpeed
					}
				},

				model::DesiredFanMode::Manual => match state.desired_manual_fan_level {
					model::DesiredManualFanLevel::Firmware(fan_firmware_level) => acpi::FanLevel::Firmware(fan_firmware_level),
					model::DesiredManualFanLevel::FullSpeed => acpi::FanLevel::FullSpeed,
				},
			};

			acpi::write_fan(fan_level).unwrap();
		}

		let temps_view_contents = match render_temps(&mut state) {
			Ok(temps_view_contents) => temps_view_contents,
			Err(err) => {
				window.quit();
				return Err(err);
			},
		};
		window.call_on_id(TEMPS_VIEW_ID, |temps_view: &mut cursive::views::StackView| {
			temps_view.pop_layer();
			temps_view.add_fullscreen_layer(temps_view_contents);
		}).unwrap();

		let fan_view_contents = match render_fan(&mut state) {
			Ok(fan_view_contents) => fan_view_contents,
			Err(err) => {
				window.quit();
				return Err(err);
			},
		};
		window.call_on_id(FAN_VIEW_ID, |fan_view: &mut cursive::views::StackView| {
			fan_view.pop_layer();
			fan_view.add_fullscreen_layer(fan_view_contents);
		}).unwrap();

		window.step();

		if !window.is_running() {
			return Ok(());
		}
	}
}

const TEMPS_VIEW_ID: &'static str = "temps_view";
const FAN_VIEW_ID: &'static str = "fan_view";
const VISIBLE_TEMP_SENSORS_GROUP_ID: &'static str = "visible_temp_sensors_group";
const TEMP_SCALE_GROUP_ID: &'static str = "temp_scale_group";
const FAN_SPEED_GROUP_ID: &'static str = "fan_speed_group";
const DESIRED_MANUAL_FAN_LEVEL_ID: &'static str = "desired_manual_fan_level";

fn render(state: &model::State) -> cursive::views::LinearLayout {
	use cursive::view::Boxable;

	cursive::views::LinearLayout::horizontal()
	.child(
		cursive::views::Panel::new(
			cursive::views::LinearLayout::vertical()
			.child(cursive::views::TextView::new("Temperatures").center().full_width())
			.child(cursive::views::IdView::new(TEMPS_VIEW_ID, cursive::views::StackView::new()))
			.child({
				let mut visible_temp_sensors_group = cursive::views::RadioGroup::new();

				cursive::views::LinearLayout::horizontal()
				.child({
					let mut button = visible_temp_sensors_group.button(model::VisibleTempSensors::All, model::VisibleTempSensors::All.to_string());
					if let model::VisibleTempSensors::All = state.visible_temp_sensors {
						button.select();
					}
					button.full_width()
				})
				.child({
					let mut button = visible_temp_sensors_group.button(model::VisibleTempSensors::Active, model::VisibleTempSensors::Active.to_string());
					if let model::VisibleTempSensors::Active = state.visible_temp_sensors {
						button.select();
					}
					button.full_width()
				})
				.child(cursive::views::IdView::new(VISIBLE_TEMP_SENSORS_GROUP_ID, RadioGroupView(visible_temp_sensors_group)))
			})
			.child({
				let mut temp_scale_group = cursive::views::RadioGroup::new();

				cursive::views::LinearLayout::horizontal()
				.child({
					let mut button = temp_scale_group.button(acpi::TempScale::Celsius, acpi::TempScale::Celsius.to_string());
					if let acpi::TempScale::Celsius = state.temp_scale {
						button.select();
					}
					button.full_width()
				})
				.child({
					let mut button = temp_scale_group.button(acpi::TempScale::Fahrenheit, acpi::TempScale::Fahrenheit.to_string());
					if let acpi::TempScale::Fahrenheit = state.temp_scale {
						button.select();
					}
					button.full_width()
				})
				.child(cursive::views::IdView::new(TEMP_SCALE_GROUP_ID, RadioGroupView(temp_scale_group)))
			})))
	.child(
		cursive::views::Panel::new(
			cursive::views::LinearLayout::vertical()
			.child(cursive::views::TextView::new("Fan").center().full_width())
			.child(cursive::views::IdView::new(FAN_VIEW_ID, cursive::views::StackView::new()))
			.child(
				cursive::views::LinearLayout::horizontal()
				.child(cursive::views::TextView::new("Mode").full_width())
				.child({
					let mut fan_speed_group = cursive::views::RadioGroup::new();

					cursive::views::LinearLayout::vertical()
					.child({
						let mut button = fan_speed_group.button(model::DesiredFanMode::Bios, model::DesiredFanMode::Bios.to_string());
						if let model::DesiredFanMode::Bios = state.desired_fan_mode {
							button.select();
						}
						button.set_enabled(state.fan_is_writable);
						button
					})
					.child({
						let mut button = fan_speed_group.button(model::DesiredFanMode::Smart, model::DesiredFanMode::Smart.to_string());
						if let model::DesiredFanMode::Smart = state.desired_fan_mode {
							button.select();
						}
						button.set_enabled(state.fan_is_writable);
						button
					})
					.child(
						cursive::views::LinearLayout::horizontal()
						.child({
							let mut button = fan_speed_group.button(model::DesiredFanMode::Manual, model::DesiredFanMode::Manual.to_string());
							if let model::DesiredFanMode::Manual = state.desired_fan_mode {
								button.select();
							}
							button.set_enabled(state.fan_is_writable);
							button
						})
						.child({
							let all_desired_manual_fan_levels = [
								model::DesiredManualFanLevel::Firmware(acpi::FanFirmwareLevel::Zero),
								model::DesiredManualFanLevel::Firmware(acpi::FanFirmwareLevel::One),
								model::DesiredManualFanLevel::Firmware(acpi::FanFirmwareLevel::Two),
								model::DesiredManualFanLevel::Firmware(acpi::FanFirmwareLevel::Three),
								model::DesiredManualFanLevel::Firmware(acpi::FanFirmwareLevel::Four),
								model::DesiredManualFanLevel::Firmware(acpi::FanFirmwareLevel::Five),
								model::DesiredManualFanLevel::Firmware(acpi::FanFirmwareLevel::Six),
								model::DesiredManualFanLevel::Firmware(acpi::FanFirmwareLevel::Seven),
								model::DesiredManualFanLevel::FullSpeed,
							];

							let mut view =
								cursive::views::SelectView::new()
								.popup()
								.with_all(all_desired_manual_fan_levels.iter().map(|&desired_manual_fan_level|
									(desired_manual_fan_level.to_string(), desired_manual_fan_level)));
							view.set_selection(all_desired_manual_fan_levels.iter().position(|v| v == &state.desired_manual_fan_level).unwrap());
							view.set_enabled(state.fan_is_writable);
							cursive::views::IdView::new(DESIRED_MANUAL_FAN_LEVEL_ID, view)
						}))
					.child(cursive::views::IdView::new(FAN_SPEED_GROUP_ID, RadioGroupView(fan_speed_group)))
				}))))
}

fn render_temps(state: &mut model::State) -> Result<cursive::views::BoxView<cursive::views::ListView>, Error> {
	use cursive::view::Boxable;

	if let Ok(temps) = &state.temps {
		Ok(
			state.config.sensors.iter().zip(temps).fold(
				cursive::views::ListView::new(),
				|layout, (name, temp)| match (name.as_ref(), temp, &state.visible_temp_sensors) {
					(Some(name), Some(temp), _) =>
						layout
						.child(name, cursive::views::TextView::new(temp.display(state.temp_scale).to_string()).h_align(cursive::align::HAlign::Right).full_width()),
					(Some(name), None, &model::VisibleTempSensors::All) =>
						layout
						.child(name, cursive::views::TextView::new("n/a").h_align(cursive::align::HAlign::Right).full_width()),
					(None, _, _) |
					(_, None, &model::VisibleTempSensors::Active) =>
						layout,
				},
			)
			.full_screen())
	}
	else {
		let err = std::mem::replace(&mut state.temps, Ok(vec![])).unwrap_err();
		return Err(err);
	}
}

fn render_fan(state: &mut model::State) -> Result<cursive::views::LinearLayout, Error> {
	use cursive::view::Boxable;

	if let Ok((fan_level, fan_speed)) = &state.fan {
		Ok(
			cursive::views::LinearLayout::vertical()
			.child(
				cursive::views::LinearLayout::horizontal()
				.child(cursive::views::TextView::new("Level").full_width())
				.child(cursive::views::TextView::new(fan_level.to_string()))
				.full_height())
			.child(
				cursive::views::LinearLayout::horizontal()
				.child(cursive::views::TextView::new("Speed").full_width())
				.child(cursive::views::TextView::new(fan_speed.to_string()))
				.full_height()))
	}
	else {
		let err = std::mem::replace(&mut state.fan, Ok((acpi::FanLevel::Auto, acpi::FanSpeed(0)))).unwrap_err();
		return Err(err);
	}
}

struct RadioGroupView<T>(cursive::views::RadioGroup<T>);

impl<T> cursive::view::View for RadioGroupView<T> where T: 'static {
	fn draw(&self, _: &cursive::Printer<'_, '_>) {
	}
}
