use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};
use ratatui::Frame;

use crate::components::{Action, Component};
use crate::domain::patient::{Patient, PatientService};
use crate::error::Result;
use crate::ui::keybinds::{KeybindContext, KeybindRegistry};
use crate::ui::widgets::HelpModal;
use std::sync::Arc;

pub struct PatientListComponent {
    patient_service: Option<Arc<PatientService>>,
    all_patients: Vec<Patient>,
    filtered_patients: Vec<Patient>,
    table_state: TableState,
    #[allow(dead_code)]
    is_loading: bool,
    error_message: Option<String>,
    search_query: String,
    search_mode: bool,
    #[allow(dead_code)]
    page: usize,
    #[allow(dead_code)]
    page_size: usize,
    showing_help_modal: bool,
}

impl PatientListComponent {
    pub fn new(patient_service: Arc<PatientService>) -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));

        Self {
            patient_service: Some(patient_service),
            all_patients: Vec::new(),
            filtered_patients: Vec::new(),
            table_state,
            is_loading: false,
            error_message: None,
            search_query: String::new(),
            search_mode: false,
            page: 0,
            page_size: 20,
            showing_help_modal: false,
        }
    }

    fn select_next(&mut self) {
        if self.filtered_patients.is_empty() {
            return;
        }

        let current = self.table_state.selected().unwrap_or(0);
        let next = (current + 1).min(self.filtered_patients.len() - 1);
        self.table_state.select(Some(next));
    }

    fn select_previous(&mut self) {
        if self.filtered_patients.is_empty() {
            return;
        }

        let current = self.table_state.selected().unwrap_or(0);
        let prev = current.saturating_sub(1);
        self.table_state.select(Some(prev));
    }

    fn select_first(&mut self) {
        if !self.filtered_patients.is_empty() {
            self.table_state.select(Some(0));
        }
    }

    fn select_last(&mut self) {
        if !self.filtered_patients.is_empty() {
            self.table_state
                .select(Some(self.filtered_patients.len() - 1));
        }
    }

    fn selected_patient(&self) -> Option<&Patient> {
        self.table_state
            .selected()
            .and_then(|i| self.filtered_patients.get(i))
    }

    fn apply_search_filter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_patients = self.all_patients.clone();
        } else {
            let query = self.search_query.to_lowercase();
            self.filtered_patients = self
                .all_patients
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

                    full_name.contains(&query)
                        || preferred.contains(&query)
                        || medicare.contains(&query)
                })
                .cloned()
                .collect();
        }

        if !self.filtered_patients.is_empty() {
            self.table_state.select(Some(0));
        } else {
            self.table_state.select(None);
        }
    }

    fn enter_search_mode(&mut self) {
        self.search_mode = true;
        self.search_query.clear();
    }

    fn exit_search_mode(&mut self) {
        self.search_mode = false;
    }

    fn handle_search_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.apply_search_filter();
                Action::Render
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.apply_search_filter();
                Action::Render
            }
            KeyCode::Enter | KeyCode::Esc => {
                self.exit_search_mode();
                Action::Render
            }
            _ => Action::None,
        }
    }

    fn render_search_bar_static(
        frame: &mut Frame,
        area: Rect,
        search_query: &str,
        search_mode: bool,
    ) {
        use ratatui::widgets::Paragraph;

        let search_text = if search_mode {
            format!("Search: {}█", search_query)
        } else {
            format!("Filter: {} (/ to edit, Esc to clear)", search_query)
        };

        let search_style = if search_mode {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Green)
        };

        let search_bar = Paragraph::new(search_text)
            .style(search_style)
            .block(Block::default().borders(Borders::ALL).title(" Search "));

        frame.render_widget(search_bar, area);
    }
}

#[async_trait]
impl Component for PatientListComponent {
    async fn init(&mut self) -> Result<()> {
        if let Some(service) = &self.patient_service {
            match service.list_active_patients().await {
                Ok(patients) => {
                    self.all_patients = patients.clone();
                    self.filtered_patients = patients;
                    self.error_message = None;

                    if !self.filtered_patients.is_empty() {
                        self.table_state.select(Some(0));
                    }
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to load patients: {}", e));
                }
            }
        }
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        if self.showing_help_modal {
            match key.code {
                KeyCode::Esc | KeyCode::Char('?') => {
                    self.showing_help_modal = false;
                    return Action::Render;
                }
                _ => return Action::None,
            }
        }

        if self.search_mode {
            return self.handle_search_input(key);
        }

        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.select_next();
                Action::Render
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.select_previous();
                Action::Render
            }
            KeyCode::Char('g') => {
                self.select_first();
                Action::Render
            }
            KeyCode::Char('G') => {
                self.select_last();
                Action::Render
            }
            KeyCode::Enter => {
                // TODO: Implement patient detail view
                // When implemented, this will navigate to a read-only patient detail screen
                // showing complete patient information, medical history, and clinical notes.
                // For now, this keybind is disabled to avoid confusion.
                if let Some(_patient) = self.selected_patient() {
                    Action::None
                } else {
                    Action::None
                }
            }
            KeyCode::Char('n') => Action::PatientCreate,
            KeyCode::Char('/') => {
                self.enter_search_mode();
                Action::Render
            }
            KeyCode::Char('?') => {
                self.showing_help_modal = true;
                Action::Render
            }
            KeyCode::Esc => {
                if !self.search_query.is_empty() {
                    self.search_query.clear();
                    self.apply_search_filter();
                    Action::Render
                } else {
                    Action::None
                }
            }
            _ => Action::None,
        }
    }

    async fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => Ok(None),
            _ => Ok(None),
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        use ratatui::layout::{Constraint as LayoutConstraint, Direction, Layout};

        let table_area = if self.search_mode || !self.search_query.is_empty() {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([LayoutConstraint::Length(3), LayoutConstraint::Min(0)])
                .split(area);
            Self::render_search_bar_static(frame, chunks[0], &self.search_query, self.search_mode);
            chunks[1]
        } else {
            area
        };

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
            .filtered_patients
            .iter()
            .map(|patient| {
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

                Row::new(vec![
                    Cell::from(name),
                    Cell::from(dob),
                    Cell::from(age),
                    Cell::from(medicare),
                    Cell::from(phone),
                ])
                .height(1)
            })
            .collect();

        let widths = [
            Constraint::Percentage(30),
            Constraint::Percentage(15),
            Constraint::Percentage(10),
            Constraint::Percentage(25),
            Constraint::Percentage(20),
        ];

        let title = if self.search_query.is_empty() {
            let help = KeybindRegistry::get_help_text(KeybindContext::PatientList);
            format!(" Patients - {} ", help)
        } else {
            format!(
                " Patients - {} results (n: New, Esc: Clear) ",
                self.filtered_patients.len()
            )
        };

        let table = Table::new(rows, widths)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title(title))
            .row_highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(table, table_area, &mut self.table_state);

        if let Some(ref error) = self.error_message {
            let error_text = format!("Error: {}", error);
            let error_paragraph =
                ratatui::widgets::Paragraph::new(error_text).style(Style::default().fg(Color::Red));
            frame.render_widget(error_paragraph, table_area);
        }

        if self.showing_help_modal {
            let help_modal = HelpModal::new(KeybindContext::PatientList);
            help_modal.render(frame, area);
        }
    }
}
