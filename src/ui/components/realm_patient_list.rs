use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState};
use tuirealm::{
    command::{Cmd, CmdResult, Direction},
    event::{Key, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind},
    AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, Props, State, StateValue,
};

use crate::domain::patient::Patient;
use crate::ui::keybinds::{Keybind, KeybindContext, KeybindRegistry};
use crate::ui::msg::Msg;
use crate::ui::theme::Theme;

#[derive(MockComponent, Clone)]
pub struct RealmPatientList {
    component: PatientListWidget,
    keybinds: Vec<Keybind>,
    all_patients: Vec<Patient>,
    filtered_patients: Vec<Patient>,
    search_query: String,
    search_mode: bool,
}

pub struct RealmPatientListBuilder {
    patients: Vec<Patient>,
    selected_index: usize,
    keybinds: Option<Vec<Keybind>>,
    search_mode: bool,
    search_query: String,
}

impl RealmPatientListBuilder {
    pub fn new() -> Self {
        Self {
            patients: Vec::new(),
            selected_index: 0,
            keybinds: None,
            search_mode: false,
            search_query: String::new(),
        }
    }

    pub fn patients(mut self, patients: Vec<Patient>) -> Self {
        self.patients = patients;
        self
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected_index = index;
        self
    }

    pub fn with_keybinds(mut self) -> Self {
        self.keybinds = Some(KeybindRegistry::get_keybinds(KeybindContext::PatientList));
        self
    }

    pub fn build(self) -> RealmPatientList {
        let theme = Theme::new();
        let list = PatientListWidget::default()
            .patients(&self.patients)
            .selected(self.selected_index)
            .normal_style(theme.normal)
            .selected_style(theme.selected)
            .highlight_style(theme.highlight);

        let keybinds = self
            .keybinds
            .unwrap_or_else(|| KeybindRegistry::get_keybinds(KeybindContext::PatientList));

        let filtered = if self.search_query.is_empty() {
            self.patients.clone()
        } else {
            Self::filter_patients(&self.patients, &self.search_query)
        };

        RealmPatientList {
            component: list,
            keybinds,
            all_patients: self.patients,
            filtered_patients: filtered,
            search_mode: self.search_mode,
            search_query: self.search_query,
        }
    }

    fn filter_patients(patients: &[Patient], query: &str) -> Vec<Patient> {
        if query.is_empty() {
            return patients.to_vec();
        }

        let query_lower = query.to_lowercase();
        patients
            .iter()
            .filter(|p| {
                let full_name = format!("{} {}", p.first_name, p.last_name).to_lowercase();
                let preferred = p
                    .preferred_name
                    .as_ref()
                    .map(|n| n.to_lowercase())
                    .unwrap_or_default();
                let medicare = p
                    .medicare_number
                    .as_ref()
                    .map(|m| m.to_lowercase())
                    .unwrap_or_default();

                full_name.contains(&query_lower)
                    || preferred.contains(&query_lower)
                    || medicare.contains(&query_lower)
            })
            .cloned()
            .collect()
    }
}

impl Default for RealmPatientListBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RealmPatientList {
    pub fn new() -> Self {
        Self::builder().build()
    }

    pub fn builder() -> RealmPatientListBuilder {
        RealmPatientListBuilder::new()
    }

    pub fn selected_patient(&self) -> Option<&Patient> {
        self.component
            .selected
            .and_then(|i| self.filtered_patients.get(i))
    }

    pub fn keybinds(&self) -> &[Keybind] {
        &self.keybinds
    }

    pub fn patients(&self) -> &[Patient] {
        &self.filtered_patients
    }

    pub fn update_patients(&mut self, patients: Vec<Patient>) {
        self.all_patients = patients.clone();
        self.apply_filter();
    }

    pub fn is_search_mode(&self) -> bool {
        self.search_mode
    }

    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    pub fn render(&mut self, frame: &mut tuirealm::Frame, area: Rect) {
        self.component.view(frame, area);
    }

    fn apply_filter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_patients = self.all_patients.clone();
        } else {
            self.filtered_patients =
                RealmPatientListBuilder::filter_patients(&self.all_patients, &self.search_query);
        }

        if !self.filtered_patients.is_empty() {
            self.component.selected = Some(0);
        } else {
            self.component.selected = None;
        }
    }
}

impl Default for RealmPatientList {
    fn default() -> Self {
        Self::new()
    }
}

impl Component<Msg, NoUserEvent> for RealmPatientList {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(key_event) => self.handle_keyboard(key_event),
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(_),
                column,
                row,
                ..
            }) => self.handle_click(column, row),
            _ => None,
        }
    }
}

impl RealmPatientList {
    pub fn handle_keyboard(&mut self, key_event: KeyEvent) -> Option<Msg> {
        let key = key_event.code;
        let modifiers = key_event.modifiers;

        if self.search_mode {
            return self.handle_search_input(key);
        }

        for kb in &self.keybinds {
            let key_match = self.key_matches(kb.key, key);

            let mod_match = if matches!(kb.key, crossterm::event::KeyCode::BackTab) {
                true
            } else {
                self.modifiers_match(kb.modifiers, modifiers)
            };

            if key_match && mod_match {
                return self.execute_action(kb.action);
            }
        }
        None
    }

    fn key_matches(&self, crossterm_key: crossterm::event::KeyCode, tui_key: Key) -> bool {
        use crossterm::event::KeyCode;
        match (crossterm_key, tui_key) {
            (KeyCode::Char(c1), Key::Char(c2)) => c1 == c2,
            (KeyCode::Enter, Key::Enter) => true,
            (KeyCode::Esc, Key::Esc) => true,
            (KeyCode::Up, Key::Up) => true,
            (KeyCode::Down, Key::Down) => true,
            (KeyCode::Backspace, Key::Backspace) => true,
            (KeyCode::Char('/'), Key::Char('/')) => true,
            _ => false,
        }
    }

    fn modifiers_match(
        &self,
        crossterm_mods: crossterm::event::KeyModifiers,
        tui_mods: KeyModifiers,
    ) -> bool {
        let has_shift =
            |m: crossterm::event::KeyModifiers| m.contains(crossterm::event::KeyModifiers::SHIFT);
        let has_ctrl =
            |m: crossterm::event::KeyModifiers| m.contains(crossterm::event::KeyModifiers::CONTROL);
        let has_alt =
            |m: crossterm::event::KeyModifiers| m.contains(crossterm::event::KeyModifiers::ALT);

        let tui_has_shift = tui_mods.intersects(KeyModifiers::SHIFT);
        let tui_has_ctrl = tui_mods.intersects(KeyModifiers::CONTROL);
        let tui_has_alt = tui_mods.intersects(KeyModifiers::ALT);

        has_shift(crossterm_mods) == tui_has_shift
            && has_ctrl(crossterm_mods) == tui_has_ctrl
            && has_alt(crossterm_mods) == tui_has_alt
    }

    fn handle_search_input(&mut self, key: Key) -> Option<Msg> {
        match key {
            Key::Char(c) => {
                self.search_query.push(c);
                self.apply_filter();
                Some(Msg::Render)
            }
            Key::Backspace => {
                self.search_query.pop();
                self.apply_filter();
                Some(Msg::Render)
            }
            Key::Enter | Key::Esc => {
                self.search_mode = false;
                Some(Msg::Render)
            }
            _ => None,
        }
    }

    fn execute_action(&mut self, action: &str) -> Option<Msg> {
        match action {
            "Next" => {
                self.move_selection(1);
                Some(Msg::Render)
            }
            "Previous" => {
                self.move_selection(-1);
                Some(Msg::Render)
            }
            "First" => {
                if !self.filtered_patients.is_empty() {
                    self.component.selected = Some(0);
                }
                Some(Msg::Render)
            }
            "Last" => {
                if !self.filtered_patients.is_empty() {
                    self.component.selected = Some(self.filtered_patients.len() - 1);
                }
                Some(Msg::Render)
            }
            "View" => {
                if let Some(patient) = self.selected_patient() {
                    Some(Msg::PatientSelected(patient.id))
                } else {
                    None
                }
            }
            "Edit" => {
                if let Some(patient) = self.selected_patient() {
                    Some(Msg::PatientEdit(patient.id))
                } else {
                    None
                }
            }
            "New" => Some(Msg::PatientCreate),
            "Search" => {
                self.search_mode = true;
                self.search_query.clear();
                Some(Msg::Render)
            }
            "Clear" => {
                if !self.search_query.is_empty() {
                    self.search_query.clear();
                    self.apply_filter();
                } else {
                    self.search_mode = false;
                }
                Some(Msg::Render)
            }
            _ => None,
        }
    }

    fn move_selection(&mut self, delta: isize) {
        if self.filtered_patients.is_empty() {
            return;
        }

        let current = self.component.selected.unwrap_or(0);
        let new = if delta > 0 {
            ((current as isize + delta) as usize).min(self.filtered_patients.len() - 1)
        } else {
            current.saturating_sub((-delta) as usize)
        };
        self.component.selected = Some(new);
    }

    fn handle_click(&mut self, column: u16, row: u16) -> Option<Msg> {
        let area = self.component.last_area;

        let inner_area = Rect {
            x: area.x + 1,
            y: area.y + 2,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(3),
        };

        if row < inner_area.y || row >= inner_area.y + inner_area.height {
            return None;
        }

        let row_index = (row - inner_area.y) as usize;
        if row_index < self.filtered_patients.len() {
            self.component.selected = Some(row_index);
            return Some(Msg::Render);
        }

        None
    }
}

#[derive(Default, Clone)]
struct PatientListWidget {
    props: Props,
    patients: Vec<Patient>,
    selected: Option<usize>,
    normal_style: Style,
    selected_style: Style,
    highlight_style: Style,
    last_area: Rect,
}

impl PatientListWidget {
    pub fn patients(mut self, patients: &[Patient]) -> Self {
        self.patients = patients.to_vec();
        self
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected = Some(index);
        self
    }

    pub fn normal_style(mut self, style: Style) -> Self {
        self.normal_style = style;
        self
    }

    pub fn selected_style(mut self, style: Style) -> Self {
        self.selected_style = style;
        self
    }

    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }

    fn get_focus(&self) -> bool {
        self.props
            .get_or(Attribute::Focus, AttrValue::Flag(false))
            .unwrap_flag()
    }
}

impl MockComponent for PatientListWidget {
    fn view(&mut self, frame: &mut tuirealm::Frame, area: Rect) {
        self.last_area = area;

        if self.patients.is_empty() {
            let block = Block::default().borders(Borders::ALL).title(" Patients ");
            let inner = block.inner(area);
            frame.render_widget(block, area);

            let empty_msg = ratatui::widgets::Paragraph::new(
                "No patients found.\n\nPress 'n' to add a new patient.",
            )
            .style(Style::default().fg(Color::Gray))
            .alignment(ratatui::layout::Alignment::Center);
            frame.render_widget(empty_msg, inner);
            return;
        }

        let focus = self.get_focus();

        let header_style = Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD);

        let header = Row::new(vec![
            Cell::from("Name"),
            Cell::from("DOB"),
            Cell::from("Age"),
            Cell::from("Medicare"),
            Cell::from("Phone"),
        ])
        .style(header_style)
        .height(1);

        let rows: Vec<Row> = self
            .patients
            .iter()
            .enumerate()
            .map(|(i, patient)| {
                let is_selected = self.selected == Some(i);
                let name = format!(
                    "{}, {}",
                    patient.last_name,
                    patient
                        .preferred_name
                        .as_ref()
                        .unwrap_or(&patient.first_name)
                );
                let dob = patient.date_of_birth.format("%d/%m/%Y").to_string();
                let age = patient.age().to_string();
                let medicare = patient
                    .medicare_number
                    .as_ref()
                    .map(|m| {
                        if let Some(irn) = patient.medicare_irn {
                            format!("{}-{}", m, irn)
                        } else {
                            m.clone()
                        }
                    })
                    .unwrap_or_else(|| "-".to_string());
                let phone = patient
                    .phone_mobile
                    .as_ref()
                    .or(patient.phone_home.as_ref())
                    .cloned()
                    .unwrap_or_else(|| "-".to_string());

                let style = if is_selected {
                    self.selected_style.add_modifier(Modifier::BOLD)
                } else if focus {
                    self.highlight_style
                } else {
                    self.normal_style
                };

                Row::new(vec![
                    Cell::from(name),
                    Cell::from(dob),
                    Cell::from(age),
                    Cell::from(medicare),
                    Cell::from(phone),
                ])
                .style(style)
                .height(1)
            })
            .collect();

        use ratatui::layout::Constraint as LayoutConstraint;
        let widths = [
            LayoutConstraint::Percentage(30),
            LayoutConstraint::Percentage(15),
            LayoutConstraint::Percentage(10),
            LayoutConstraint::Percentage(25),
            LayoutConstraint::Percentage(20),
        ];

        let table = Table::new(rows, widths)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title(" Patients "))
            .row_highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        let mut table_state = TableState::default();
        table_state.select(self.selected);

        frame.render_stateful_widget(table, area, &mut table_state);

        self.selected = table_state.selected();
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.props.get(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.props.set(attr, value);
    }

    fn state(&self) -> State {
        State::One(StateValue::Usize(self.selected.unwrap_or(0)))
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        match cmd {
            Cmd::Move(Direction::Up) => {
                if let Some(selected) = self.selected {
                    let new = selected.saturating_sub(1);
                    self.selected = Some(new);
                    return CmdResult::Changed(State::One(StateValue::Usize(new)));
                }
                CmdResult::None
            }
            Cmd::Move(Direction::Down) => {
                if let Some(selected) = self.selected {
                    let new = (selected + 1).min(self.patients.len().saturating_sub(1));
                    self.selected = Some(new);
                    return CmdResult::Changed(State::One(StateValue::Usize(new)));
                }
                CmdResult::None
            }
            Cmd::GoTo(tuirealm::command::Position::Begin) => {
                if !self.patients.is_empty() {
                    self.selected = Some(0);
                    return CmdResult::Changed(State::One(StateValue::Usize(0)));
                }
                CmdResult::None
            }
            Cmd::GoTo(tuirealm::command::Position::End) => {
                if !self.patients.is_empty() {
                    let last = self.patients.len() - 1;
                    self.selected = Some(last);
                    return CmdResult::Changed(State::One(StateValue::Usize(last)));
                }
                CmdResult::None
            }
            _ => CmdResult::None,
        }
    }
}
