use chrono::NaiveDate;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table};
use tuirealm::{
    command::{Cmd, CmdResult, Direction as TuiDirection},
    event::{Key, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind},
    AttrValue, Attribute, Component, Event, MockComponent, NoUserEvent, Props, State, StateValue,
};

use crate::domain::patient::Patient;
use crate::ui::keybinds::{Keybind, KeybindContext, KeybindRegistry};
use crate::ui::msg::Msg;
use crate::ui::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum FormField {
    FirstName,
    LastName,
    DateOfBirth,
    Gender,
    MedicareNumber,
    MedicareIrn,
    PhoneMobile,
    Email,
}

impl FormField {
    fn all() -> Vec<FormField> {
        vec![
            FormField::FirstName,
            FormField::LastName,
            FormField::DateOfBirth,
            FormField::Gender,
            FormField::MedicareNumber,
            FormField::MedicareIrn,
            FormField::PhoneMobile,
            FormField::Email,
        ]
    }

    fn label(&self) -> &str {
        match self {
            FormField::FirstName => "First Name *",
            FormField::LastName => "Last Name *",
            FormField::DateOfBirth => "Date of Birth * (DD/MM/YYYY)",
            FormField::Gender => "Gender *",
            FormField::MedicareNumber => "Medicare Number",
            FormField::MedicareIrn => "Medicare IRN (1-9)",
            FormField::PhoneMobile => "Mobile Phone",
            FormField::Email => "Email",
        }
    }
}

#[derive(MockComponent, Clone)]
pub struct RealmPatientForm {
    component: FormWidget,
    keybinds: Vec<Keybind>,
    editing_patient_id: Option<String>,
    current_field: usize,
    scroll_offset: usize,
    validation_errors: Vec<String>,
    is_submitting: bool,
}

pub struct RealmPatientFormBuilder {
    patient: Option<Patient>,
    keybinds: Option<Vec<Keybind>>,
    editing: bool,
}

impl RealmPatientFormBuilder {
    pub fn new() -> Self {
        Self {
            patient: None,
            keybinds: None,
            editing: false,
        }
    }

    pub fn patient(mut self, patient: Patient) -> Self {
        self.patient = Some(patient);
        self.editing = true;
        self
    }

    pub fn with_keybinds(mut self) -> Self {
        self.keybinds = Some(KeybindRegistry::get_keybinds(KeybindContext::PatientForm));
        self
    }

    pub fn build(self) -> RealmPatientForm {
        let theme = Theme::new();
        let form = FormWidget::default()
            .normal_style(theme.normal)
            .selected_style(theme.selected)
            .highlight_style(theme.highlight);

        let keybinds = self
            .keybinds
            .unwrap_or_else(|| KeybindRegistry::get_keybinds(KeybindContext::PatientForm));

        let editing_patient_id = self.patient.as_ref().map(|p| p.id.to_string());

        RealmPatientForm {
            component: form,
            keybinds,
            editing_patient_id,
            current_field: 0,
            scroll_offset: 0,
            validation_errors: Vec::new(),
            is_submitting: false,
        }
    }
}

impl Default for RealmPatientFormBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RealmPatientForm {
    pub fn new() -> Self {
        Self::builder().build()
    }

    pub fn builder() -> RealmPatientFormBuilder {
        RealmPatientFormBuilder::new()
    }

    pub fn keybinds(&self) -> &[Keybind] {
        &self.keybinds
    }

    pub fn is_edit_mode(&self) -> bool {
        self.editing_patient_id.is_some()
    }

    pub fn get_form_data(&self) -> Option<FormData> {
        let field = FormField::all()[self.current_field];
        let value = self.component.get_field_value(&field);

        Some(FormData {
            first_name: self.component.get_field_value(&FormField::FirstName),
            last_name: self.component.get_field_value(&FormField::LastName),
            date_of_birth: self.component.get_field_value(&FormField::DateOfBirth),
            gender: self.component.get_gender_index(),
            medicare_number: self.component.get_field_value(&FormField::MedicareNumber),
            medicare_irn: self.component.get_field_value(&FormField::MedicareIrn),
            phone_mobile: self.component.get_field_value(&FormField::PhoneMobile),
            email: self.component.get_field_value(&FormField::Email),
            current_field: field,
            current_value: value,
        })
    }

    fn validate(&self) -> bool {
        let mut errors = Vec::new();

        let first_name = self.component.get_field_value(&FormField::FirstName);
        if first_name.trim().is_empty() {
            errors.push("First name is required".to_string());
        }

        let last_name = self.component.get_field_value(&FormField::LastName);
        if last_name.trim().is_empty() {
            errors.push("Last name is required".to_string());
        }

        let dob = self.component.get_field_value(&FormField::DateOfBirth);
        if dob.trim().is_empty() {
            errors.push("Date of birth is required".to_string());
        } else if self.parse_date(&dob).is_none() {
            errors.push("Date of birth must be in DD/MM/YYYY format".to_string());
        }

        let medicare = self.component.get_field_value(&FormField::MedicareNumber);
        if !medicare.is_empty() && medicare.len() != 10 {
            errors.push("Medicare number must be 10 digits".to_string());
        }

        let irn = self.component.get_field_value(&FormField::MedicareIrn);
        if !irn.is_empty() {
            if let Ok(irn_val) = irn.parse::<u8>() {
                if !(1..=9).contains(&irn_val) {
                    errors.push("Medicare IRN must be between 1 and 9".to_string());
                }
            } else {
                errors.push("Medicare IRN must be a number".to_string());
            }
        }

        errors.is_empty()
    }

    fn parse_date(&self, date_str: &str) -> Option<NaiveDate> {
        let parts: Vec<&str> = date_str.split('/').collect();
        if parts.len() != 3 {
            return None;
        }

        let day = parts[0].parse::<u32>().ok()?;
        let month = parts[1].parse::<u32>().ok()?;
        let year = parts[2].parse::<i32>().ok()?;

        NaiveDate::from_ymd_opt(year, month, day)
    }

    fn current_field(&self) -> FormField {
        FormField::all()[self.current_field]
    }

    fn next_field(&mut self) {
        let fields = FormField::all();
        self.current_field = (self.current_field + 1) % fields.len();
        self.adjust_scroll();
    }

    fn prev_field(&mut self) {
        let fields = FormField::all();
        if self.current_field == 0 {
            self.current_field = fields.len() - 1;
        } else {
            self.current_field -= 1;
        }
        self.adjust_scroll();
    }

    fn adjust_scroll(&mut self) {
        const VISIBLE_FIELDS: usize = 6;

        if self.current_field < self.scroll_offset {
            self.scroll_offset = self.current_field;
        } else if self.current_field >= self.scroll_offset + VISIBLE_FIELDS {
            self.scroll_offset = self.current_field.saturating_sub(VISIBLE_FIELDS - 1);
        }
    }
}

impl Default for RealmPatientForm {
    fn default() -> Self {
        Self::new()
    }
}

impl Component<Msg, NoUserEvent> for RealmPatientForm {
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

impl RealmPatientForm {
    fn handle_keyboard(&mut self, key_event: KeyEvent) -> Option<Msg> {
        if self.is_submitting {
            return None;
        }

        let key = key_event.code;
        let modifiers = key_event.modifiers;

        let is_ctrl_s =
            matches!(key, Key::Char('s')) && modifiers.intersects(KeyModifiers::CONTROL);

        if is_ctrl_s {
            if self.validate() {
                self.is_submitting = true;
                if let Some(patient_id_str) = self.editing_patient_id.as_ref() {
                    if let Ok(patient_id) = uuid::Uuid::parse_str(patient_id_str) {
                        return Some(Msg::PatientFormSubmit(patient_id));
                    }
                }
                return Some(Msg::PatientCreate);
            } else {
                self.component
                    .set_validation_errors(vec!["Fix validation errors first".to_string()]);
                return Some(Msg::Render);
            }
        }

        if modifiers.intersects(KeyModifiers::SHIFT) && matches!(key, Key::Tab) {
            self.prev_field();
            return Some(Msg::Render);
        }

        if matches!(key, Key::Tab) {
            self.next_field();
            return Some(Msg::Render);
        }

        if matches!(key, Key::Esc) {
            return Some(Msg::PatientFormCancel);
        }

        if matches!(key, Key::Up) {
            if matches!(self.current_field(), FormField::Gender) {
                self.component.prev_gender();
            }
            self.prev_field();
            return Some(Msg::Render);
        }

        if matches!(key, Key::Down) {
            if matches!(self.current_field(), FormField::Gender) {
                self.component.next_gender();
            }
            self.next_field();
            return Some(Msg::Render);
        }

        match key {
            Key::Char(ch) => {
                self.component
                    .update_field(&self.current_field(), ch.to_string());
                return Some(Msg::Render);
            }
            Key::Backspace => {
                self.component.delete_from_field(&self.current_field());
                return Some(Msg::Render);
            }
            _ => {}
        }

        None
    }

    fn handle_click(&mut self, column: u16, row: u16) -> Option<Msg> {
        let area = self.component.last_area;

        let inner_area = Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        };

        if row < inner_area.y + 2 || row >= inner_area.y + inner_area.height - 1 {
            return None;
        }

        let field_row = row - (inner_area.y + 2);
        if field_row >= 3 {
            return None;
        }

        let clicked_field_index = self.scroll_offset + field_row as usize;
        let all_fields = FormField::all();

        if clicked_field_index < all_fields.len() {
            self.current_field = clicked_field_index;
            return Some(Msg::Render);
        }

        None
    }
}

#[derive(Clone)]
pub struct FormData {
    pub first_name: String,
    pub last_name: String,
    pub date_of_birth: String,
    pub gender: usize,
    pub medicare_number: String,
    pub medicare_irn: String,
    pub phone_mobile: String,
    pub email: String,
    pub current_field: FormField,
    pub current_value: String,
}

#[derive(Default, Clone)]
struct FormWidget {
    props: Props,
    fields: std::collections::HashMap<FormField, String>,
    gender_index: usize,
    focused_field_index: usize,
    normal_style: Style,
    selected_style: Style,
    highlight_style: Style,
    last_area: Rect,
    validation_errors: Vec<String>,
}

impl FormWidget {
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

    pub fn get_field_value(&self, field: &FormField) -> String {
        self.fields.get(field).cloned().unwrap_or_default()
    }

    pub fn get_gender_index(&self) -> usize {
        self.gender_index
    }

    pub fn prev_gender(&mut self) {
        if self.gender_index > 0 {
            self.gender_index -= 1;
        }
    }

    pub fn next_gender(&mut self) {
        if self.gender_index < 3 {
            self.gender_index += 1;
        }
    }

    pub fn update_field(&mut self, field: &FormField, value: String) {
        let current = self.fields.entry(*field).or_insert_with(String::new);
        current.push_str(&value);
    }

    pub fn delete_from_field(&mut self, field: &FormField) {
        if let Some(current) = self.fields.get_mut(field) {
            current.pop();
        }
    }

    pub fn set_validation_errors(&mut self, errors: Vec<String>) {
        self.validation_errors = errors;
    }

    fn get_focus(&self) -> bool {
        self.props
            .get_or(Attribute::Focus, AttrValue::Flag(false))
            .unwrap_flag()
    }
}

impl MockComponent for FormWidget {
    fn view(&mut self, frame: &mut tuirealm::Frame, area: Rect) {
        self.last_area = area;

        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(10),
                Constraint::Percentage(80),
                Constraint::Percentage(10),
            ])
            .split(area);

        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(15),
                Constraint::Percentage(70),
                Constraint::Percentage(15),
            ])
            .split(vertical[1]);

        let modal_area = horizontal[1];
        frame.render_widget(Clear, modal_area);

        let modal_block = Block::default()
            .borders(Borders::ALL)
            .title(" Patient Form ")
            .border_style(self.normal_style);
        let inner_area = modal_block.inner(modal_area);
        frame.render_widget(modal_block, modal_area);

        let all_fields = FormField::all();

        let field_rows: Vec<Row> = all_fields
            .iter()
            .enumerate()
            .map(|(i, field)| {
                let is_current = i == self.focused_field_index;
                let value = self.fields.get(field).cloned().unwrap_or_default();

                let display_value = if field != &FormField::Gender {
                    if is_current && !value.is_empty() {
                        format!("{} █", value)
                    } else {
                        value
                    }
                } else {
                    match self.gender_index {
                        0 => "Male".to_string(),
                        1 => "Female".to_string(),
                        2 => "Other".to_string(),
                        _ => "Prefer not to say".to_string(),
                    }
                };

                let style = if is_current {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                Row::new(vec![Cell::from(field.label()), Cell::from(display_value)])
                    .style(style)
                    .height(1)
            })
            .collect();

        let table = Table::new(
            field_rows,
            [Constraint::Percentage(40), Constraint::Percentage(60)],
        )
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

        let table_area = Rect {
            x: inner_area.x,
            y: inner_area.y + 1,
            width: inner_area.width,
            height: inner_area.height.saturating_sub(4),
        };
        frame.render_widget(table, table_area);

        let help_text = format!("Field {}/{}", all_fields.len(), all_fields.len());
        let help_widget = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Green))
            .block(Block::default().borders(Borders::ALL).title(" Help "));

        frame.render_widget(help_widget, vertical[2]);
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.props.get(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.props.set(attr, value);
    }

    fn state(&self) -> State {
        State::One(StateValue::Usize(self.focused_field_index))
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        match cmd {
            Cmd::Move(TuiDirection::Up) => {
                if self.focused_field_index > 0 {
                    self.focused_field_index -= 1;
                }
                CmdResult::Changed(self.state())
            }
            Cmd::Move(TuiDirection::Down) => {
                if self.focused_field_index < FormField::all().len() - 1 {
                    self.focused_field_index += 1;
                }
                CmdResult::Changed(self.state())
            }
            _ => CmdResult::None,
        }
    }
}
