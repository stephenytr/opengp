use crate::ui::app::App;
use crate::ui::components::appointment::AppointmentView;
use crate::ui::components::status_bar::STATUS_BAR_HEIGHT;
use crate::ui::components::tabs::Tab;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;

impl App {
    pub fn render(&mut self, frame: &mut Frame) {
        let terminal = frame.area();

        if self.help_overlay.is_visible() {
            frame.render_widget(self.help_overlay.clone(), terminal);
            return;
        }

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Min(0),
                Constraint::Length(STATUS_BAR_HEIGHT),
            ])
            .split(terminal);

        let tab_bar_area = main_layout[0];
        let content_area = main_layout[1];
        let status_bar_area = main_layout[2];

        frame.render_widget(self.tab_bar.clone(), tab_bar_area);

        self.render_content(frame, content_area);

        frame.render_widget(self.status_bar.clone(), status_bar_area);

        if self.patient_list.is_searching() {
            use ratatui::prelude::{Stylize, Widget};
            use ratatui::text::Line;
            use ratatui::widgets::Clear;

            let query = self.patient_list.search_query();
            let search_text = if query.is_empty() {
                Line::from(vec!["/".bold()])
            } else {
                Line::from(vec![format!("/{}", query).into()])
            };
            let overlay_area = Rect::new(
                content_area.x + 1,
                content_area.y + 1,
                content_area.width.saturating_sub(2),
                1,
            );
            frame.render_widget(Clear, overlay_area);
            search_text.render(overlay_area, frame.buffer_mut());
        }
    }

    fn render_content(&mut self, frame: &mut Frame, area: Rect) {
        let tab = self.tab_bar.selected();

        match tab {
            Tab::Patient => self.render_patient_tab(frame, area),
            Tab::Appointment => self.render_appointment_tab(frame, area),
            Tab::Clinical => self.render_clinical_tab(frame, area),
            Tab::Billing => self.render_billing_tab(frame, area),
        }
    }

    fn render_patient_tab(&mut self, frame: &mut Frame, area: Rect) {
        if let Some(ref mut form) = self.patient_form {
            frame.render_widget(form.clone(), area);
        } else {
            frame.render_widget(self.patient_list.clone(), area);
        }
    }

    fn render_appointment_tab(&mut self, frame: &mut Frame, area: Rect) {
        use ratatui::widgets::Clear;

        if let Some(ref form) = self.appointment_form {
            frame.render_widget(form.clone(), area);
            return;
        }

        if let Some(ref modal) = self.appointment_detail_modal {
            frame.render_widget(Clear, area);
            frame.render_widget(modal.clone(), area);
            return;
        }

        match self.appointment_state.current_view {
            AppointmentView::Calendar => {
                frame.render_widget(self.appointment_state.calendar.clone(), area);
            }
            AppointmentView::Schedule => {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
                    .split(area);

                frame.render_widget(self.appointment_state.calendar.clone(), chunks[0]);

                if self.appointment_state.is_loading {
                    use ratatui::text::Text;
                    use ratatui::widgets::{Block, Borders, Paragraph};

                    let paragraph = Paragraph::new(Text::from("Loading appointments..."))
                        .block(
                            Block::default()
                                .title(" Schedule ")
                                .borders(Borders::ALL)
                                .border_style(
                                    ratatui::style::Style::default().fg(self.theme.colors.border),
                                ),
                        )
                        .alignment(ratatui::layout::Alignment::Center)
                        .style(ratatui::style::Style::default().fg(self.theme.colors.foreground));
                    frame.render_widget(paragraph, chunks[1]);
                    return;
                }

                let schedule = &mut self.appointment_state.schedule;

                let schedule_inner_height = chunks[1].height.saturating_sub(2);
                schedule.set_inner_height(schedule_inner_height);

                if let Some(ref data) = self.appointment_state.schedule_data {
                    schedule.load_schedule(data.clone());
                }

                if !self.appointment_state.practitioners.is_empty()
                    && self.appointment_state.schedule_data.is_none()
                {
                    use opengp_domain::domain::appointment::{
                        CalendarDayView, PractitionerSchedule,
                    };

                    let date = self
                        .appointment_state
                        .selected_date
                        .unwrap_or_else(|| chrono::Utc::now().date_naive());

                    let schedules: Vec<PractitionerSchedule> = self
                        .appointment_state
                        .practitioners
                        .iter()
                        .map(|p| PractitionerSchedule {
                            practitioner_id: p.id,
                            practitioner_name: p.display_name(),
                            appointments: Vec::new(),
                        })
                        .collect();

                    let day_view = CalendarDayView {
                        date,
                        practitioners: schedules,
                    };

                    schedule.load_schedule(day_view);
                }

                frame.render_widget(schedule.clone(), chunks[1]);
            }
        }
    }

    fn render_clinical_tab(&mut self, frame: &mut Frame, area: Rect) {
        use crate::ui::components::clinical::{ClinicalFormView, SocialHistoryComponent};

        if self.clinical_state.is_form_open() {
            match self.clinical_state.form_view.clone() {
                ClinicalFormView::AllergyForm => {
                    if let Some(ref form) = self.clinical_state.allergy_form {
                        frame.render_widget(form.clone(), area);
                    }
                }
                ClinicalFormView::MedicalHistoryForm => {
                    if let Some(ref form) = self.clinical_state.medical_history_form {
                        frame.render_widget(form.clone(), area);
                    }
                }
                ClinicalFormView::VitalSignsForm => {
                    if let Some(ref form) = self.clinical_state.vitals_form {
                        frame.render_widget(form.clone(), area);
                    }
                }
                ClinicalFormView::FamilyHistoryForm => {
                    if let Some(ref form) = self.clinical_state.family_history_form {
                        frame.render_widget(form.clone(), area);
                    }
                }
                ClinicalFormView::ConsultationForm => {
                    if let Some(ref form) = self.clinical_state.consultation_form {
                        frame.render_widget(form.clone(), area);
                    }
                }
                ClinicalFormView::SocialHistoryForm => {
                    if let Some(ref form) = self.clinical_state.social_history_form {
                        frame.render_widget(form.clone(), area);
                    }
                }
                ClinicalFormView::None => {}
            }
            return;
        }

        if !self.clinical_state.has_patient() {
            use ratatui::text::Text;
            use ratatui::widgets::{Block, Borders, Paragraph};

            let content =
                "No Patient Selected\n\nPlease select a patient from the Patient tab\nto view their clinical records.";

            let paragraph = Paragraph::new(Text::from(content))
                .block(
                    Block::default()
                        .title(format!(" {} ", self.tab_bar.selected().name()))
                        .borders(Borders::ALL)
                        .border_style(
                            ratatui::style::Style::default().fg(self.theme.colors.border),
                        ),
                )
                .style(ratatui::style::Style::default().fg(self.theme.colors.foreground))
                .alignment(ratatui::layout::Alignment::Center);

            frame.render_widget(paragraph, area);
            return;
        }

        self.clinical_state.consultation_list.consultations =
            self.clinical_state.consultations.clone();
        self.clinical_state.consultation_list.loading = self.clinical_state.loading;

        self.clinical_state.allergy_list.allergies = self.clinical_state.allergies.clone();
        self.clinical_state.allergy_list.loading = self.clinical_state.loading;

        self.clinical_state.medical_history_list.conditions =
            self.clinical_state.medical_history.clone();
        self.clinical_state.medical_history_list.loading = self.clinical_state.loading;

        self.clinical_state.vitals_list.vitals = self.clinical_state.vital_signs.clone();
        self.clinical_state.vitals_list.loading = self.clinical_state.loading;

        self.clinical_state.family_history_list.entries =
            self.clinical_state.family_history.clone();
        self.clinical_state.family_history_list.loading = self.clinical_state.loading;

        match self.clinical_state.view {
            crate::ui::components::clinical::ClinicalView::PatientSummary => {
                use crate::ui::components::clinical::PatientSummaryComponent;

                let patient_item = self.patient_list.selected_patient();

                let mut component = PatientSummaryComponent::new(self.theme.clone());

                component.patient = patient_item.cloned();

                component.allergies = self.clinical_state.allergies.clone();
                component.conditions = self.clinical_state.medical_history.clone();
                component.consultations = self.clinical_state.consultations.clone();
                component.vitals = self.clinical_state.vital_signs.last().cloned();

                frame.render_widget(component, area);
            }
            crate::ui::components::clinical::ClinicalView::Consultations => {
                frame.render_widget(self.clinical_state.consultation_list.clone(), area);
            }
            crate::ui::components::clinical::ClinicalView::Allergies => {
                frame.render_widget(self.clinical_state.allergy_list.clone(), area);
            }
            crate::ui::components::clinical::ClinicalView::MedicalHistory => {
                frame.render_widget(self.clinical_state.medical_history_list.clone(), area);
            }
            crate::ui::components::clinical::ClinicalView::VitalSigns => {
                frame.render_widget(self.clinical_state.vitals_list.clone(), area);
            }
            crate::ui::components::clinical::ClinicalView::SocialHistory => {
                let mut component = SocialHistoryComponent::new(self.theme.clone());
                component.loading = self.clinical_state.loading;
                if let Some(ref sh) = self.clinical_state.social_history {
                    component.social_history = Some(
                        crate::ui::components::clinical::social_history::SocialHistoryData {
                            smoking_status: sh.smoking_status,
                            cigarettes_per_day: sh.cigarettes_per_day,
                            smoking_quit_date: sh.smoking_quit_date,
                            alcohol_status: sh.alcohol_status,
                            standard_drinks_per_week: sh.standard_drinks_per_week,
                            exercise_frequency: sh.exercise_frequency,
                            occupation: sh.occupation.clone(),
                            living_situation: sh.living_situation.clone(),
                            support_network: sh.support_network.clone(),
                            notes: sh.notes.clone(),
                        },
                    );
                }
                frame.render_widget(component, area);
            }
            crate::ui::components::clinical::ClinicalView::FamilyHistory => {
                frame.render_widget(self.clinical_state.family_history_list.clone(), area);
            }
        }

        if let Some(ref mut search) = self.clinical_state.patient_search {
            if search.is_open() {
                use crate::ui::widgets::SearchableList;
                let picker = SearchableList::new(search, &self.theme, "Select Patient", true);
                frame.render_widget(picker, area);
            }
        }
    }

    fn render_billing_tab(&mut self, frame: &mut Frame, area: Rect) {
        use ratatui::text::Text;
        use ratatui::widgets::{Block, Borders, Paragraph};

        let content = "Billing\n\nInvoicing and payments\nMedicare claims";

        let paragraph = Paragraph::new(Text::from(content))
            .block(
                Block::default()
                    .title(format!(" {} ", self.tab_bar.selected().name()))
                    .borders(Borders::ALL)
                    .border_style(ratatui::style::Style::default().fg(self.theme.colors.border)),
            )
            .style(ratatui::style::Style::default().fg(self.theme.colors.foreground))
            .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(paragraph, area);
    }
}
