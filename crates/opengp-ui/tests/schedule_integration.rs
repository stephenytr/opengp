use chrono::NaiveDate;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use opengp_config::CalendarConfig;
use opengp_infrastructure::infrastructure::fixtures::schedule_scenarios::ScheduleScenario;
use opengp_ui::ui::components::appointment::state::AppointmentState;
use opengp_ui::ui::theme::Theme;
use uuid::Uuid;

fn make_key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn test_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 1, 15).unwrap()
}

fn test_pid() -> Uuid {
    Uuid::from_u128(1)
}

#[test]
fn test_navigate_down_selects_next_slot() {
    let mut state = AppointmentState::new(Theme::default(), CalendarConfig::default());
    let view = ScheduleScenario::single_appointment(test_date(), test_pid());
    state.load_schedule_data(view);
    let initial_slot = state.selected_time_slot;
    state.handle_key(make_key(KeyCode::Down));
    assert_eq!(
        state.selected_time_slot,
        initial_slot + 1,
        "Down should increment selected_time_slot"
    );
}

#[test]
fn test_navigate_right_selects_next_practitioner() {
    let mut state = AppointmentState::new(Theme::default(), CalendarConfig::default());
    let pids = vec![Uuid::from_u128(10), Uuid::from_u128(11)];
    let view = ScheduleScenario::multi_practitioner(test_date(), &pids);
    state.load_schedule_data(view);
    let initial_idx = state.selected_practitioner_index;
    state.handle_key(make_key(KeyCode::Right));
    assert_eq!(
        state.selected_practitioner_index,
        initial_idx + 1,
        "Right should increment selected_practitioner_index"
    );
}

#[test]
fn test_toggle_column_hides_practitioner() {
    let mut state = AppointmentState::new(Theme::default(), CalendarConfig::default());
    let pids = vec![
        Uuid::from_u128(20),
        Uuid::from_u128(21),
        Uuid::from_u128(22),
    ];
    let view = ScheduleScenario::multi_practitioner(test_date(), &pids);
    state.load_schedule_data(view);
    state.toggle_column(pids[0], 3);
    assert!(
        state.is_column_hidden(pids[0]),
        "First practitioner should be hidden"
    );
    assert!(
        !state.is_column_hidden(pids[1]),
        "Second practitioner should still be visible"
    );
    assert!(
        !state.is_column_hidden(pids[2]),
        "Third practitioner should still be visible"
    );
}

#[test]
fn test_navigate_up_does_not_go_below_zero() {
    let mut state = AppointmentState::new(Theme::default(), CalendarConfig::default());
    state.selected_time_slot = 0;
    state.handle_key(make_key(KeyCode::Up));
    assert_eq!(
        state.selected_time_slot, 0,
        "Up at slot 0 should not underflow"
    );
}

#[test]
fn test_navigate_left_does_not_go_below_zero() {
    let mut state = AppointmentState::new(Theme::default(), CalendarConfig::default());
    let pids = vec![Uuid::from_u128(30), Uuid::from_u128(31)];
    let view = ScheduleScenario::multi_practitioner(test_date(), &pids);
    state.load_schedule_data(view);
    state.selected_practitioner_index = 0;
    state.handle_key(make_key(KeyCode::Left));
    assert_eq!(
        state.selected_practitioner_index, 0,
        "Left at index 0 should not underflow"
    );
}

#[test]
fn test_full_morning_loads_12_appointments() {
    let mut state = AppointmentState::new(Theme::default(), CalendarConfig::default());
    let view = ScheduleScenario::full_morning(test_date(), test_pid());
    state.load_schedule_data(view);
    let total_appts: usize = state
        .schedule_data
        .as_ref()
        .map(|d| d.practitioners.iter().map(|p| p.appointments.len()).sum())
        .unwrap_or(0);
    assert_eq!(total_appts, 12, "Full morning should have 12 appointments");
}
