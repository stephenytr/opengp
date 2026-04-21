use crate::ui::app::App;
use crate::ui::components::appointment::AppointmentView;
use crate::ui::components::clinical::ClinicalView;
use crate::ui::components::clinical_row::{ClinicalMenuKind, ClinicalRow};
use crate::ui::components::patient_tab_bar::PatientTabBar;
use crate::ui::components::status_bar::STATUS_BAR_HEIGHT;
use crate::ui::components::tabs::Tab;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;
use ratatui::widgets::Widget;

impl App {
    pub fn render(&mut self, frame: &mut Frame) {
        let terminal = frame.area();

        if self.help_overlay.is_visible() {
            frame.render_widget(self.help_overlay.clone(), terminal);
            return;
        }

        if !self.authenticated {
            frame.render_widget(self.login_screen.clone(), terminal);
            return;
        }

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(STATUS_BAR_HEIGHT),
            ])
            .split(terminal);

        let tab_bar_area = main_layout[0];
        let clinical_row_area = main_layout[1];
        let content_area = main_layout[2];
        let status_bar_area = main_layout[3];

        // Split tab_bar_area (height=2) into two lines:
        // Line 1: Main TabBar (Schedule | Patient Search)
        // Line 2: PatientTabBar (open patient colour tabs)
        let [tab_bar_line_1, tab_bar_line_2] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
        ]).areas(tab_bar_area);

        // Render main TabBar on line 1
        self.tab_bar.clone().render(tab_bar_line_1, frame.buffer_mut());

        // Render PatientTabBar on line 2 (only when patients are open)
        if !self.workspace_manager.workspaces.is_empty() {
            let patient_tabs = self.workspace_manager.workspaces
                .iter()
                .map(|ws| crate::ui::components::patient_tab_bar::PatientTab::new(
                    ws.patient_id,
                    ws.patient_snapshot.full_name.clone(),
                    ws.colour,
                ))
                .collect::<Vec<_>>();

            let active_idx = self.workspace_manager.active_index.unwrap_or(0);

            let patient_tab_bar = PatientTabBar::new(
                patient_tabs,
                active_idx,
                self.theme.clone(),
            );
            patient_tab_bar.render(tab_bar_line_2, frame.buffer_mut());
        }

        if self.workspace_manager.active().is_some() {
            let active_workspace = self.workspace_manager.active().unwrap();
            let patient_colour = active_workspace.colour;
            let clinical_items = ClinicalMenuKind::all();
            let active_clinical_idx = match active_workspace.active_clinical_menu {
                ClinicalMenuKind::Consultations => 0,
                ClinicalMenuKind::Vitals => 1,
                ClinicalMenuKind::Allergies => 2,
                ClinicalMenuKind::MedicalHistory => 3,
                ClinicalMenuKind::FamilyHistory => 4,
                ClinicalMenuKind::SocialHistory => 5,
            };
            let clinical_row = ClinicalRow::new(
                clinical_items,
                active_clinical_idx,
                patient_colour,
                self.theme.clone(),
            );
            clinical_row.render(clinical_row_area, frame.buffer_mut());
        }

        self.render_content(frame, content_area);

        self.render_server_unavailable_banner(frame, content_area);

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

    fn render_server_unavailable_banner(&self, frame: &mut Frame, area: Rect) {
        let Some(error_message) = self.server_unavailable_error.as_deref() else {
            return;
        };

        use ratatui::style::Style;
        use ratatui::text::Line;
        use ratatui::widgets::{Block, Borders, Clear, Paragraph};

        let banner_width = area.width.min(80);
        let banner_height = 4;
        let banner_area = Rect::new(
            area.x + area.width.saturating_sub(banner_width) / 2,
            area.y,
            banner_width,
            banner_height,
        );

        let instructions = Line::from("[r] Retry    [Esc] Dismiss");
        let content = vec![
            Line::from(error_message.to_string()),
            Line::from(""),
            instructions,
        ];

        let widget = Paragraph::new(content)
            .block(
                Block::default()
                    .title(" Cannot connect to server ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.theme.colors.error)),
            )
            .style(Style::default().fg(self.theme.colors.error));

        frame.render_widget(Clear, banner_area);
        frame.render_widget(widget, banner_area);
    }

    fn render_content(&mut self, frame: &mut Frame, area: Rect) {
        let tab = self.tab_bar.selected();

        match tab {
            Tab::PatientSearch => {
                if let Some(workspace) = self.workspace_manager.active() {
                    self.render_workspace(frame, area, workspace.clone());
                } else {
                    self.render_welcome_panel(frame, area);
                }
            }
            Tab::Schedule => self.render_appointment_tab(frame, area),
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
        if let Some(ref form) = self.appointment_form {
            frame.render_widget(form.clone(), area);
            return;
        }

        // Render calendar/schedule FIRST (visible behind modal)
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

                if self.appointment_state.is_loading() {
                    use ratatui::widgets::{Block, Borders};

                    let mut loading_state = self.appointment_state.loading_state.clone();
                    loading_state.tick();
                    let indicator = loading_state.to_indicator(self.theme.clone());

                    let block = Block::default()
                        .title(" Schedule ")
                        .borders(Borders::ALL)
                        .border_style(
                            ratatui::style::Style::default().fg(self.theme.colors.border),
                        );

                    frame.render_widget(block, chunks[1]);

                    let inner = chunks[1].inner(ratatui::layout::Margin {
                        vertical: 1,
                        horizontal: 1,
                    });
                    frame.render_widget(indicator, inner);
                    return;
                }

                let schedule_inner_height = chunks[1].height.saturating_sub(2);
                self.appointment_state
                    .set_inner_height(schedule_inner_height);

                if let Some(data) = self.appointment_state.schedule_data.clone() {
                    self.appointment_state.load_schedule_data(data);
                }

                let schedule_has_practitioners = self
                    .appointment_state
                    .schedule_data
                    .as_ref()
                    .map(|d| !d.practitioners.is_empty())
                    .unwrap_or(false);

                if !self.appointment_state.practitioners.is_empty() && !schedule_has_practitioners {
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
                            working_hours: None,
                        })
                        .collect();

                    let day_view = CalendarDayView {
                        date,
                        practitioners: schedules,
                    };

                    self.appointment_state.load_schedule_data(day_view);
                }

                let schedule = self.appointment_state.schedule.clone();
                frame.render_stateful_widget(schedule, chunks[1], &mut self.appointment_state);
            }
        }

        // Render modal ON TOP (overlay)
        if let Some(ref modal) = self.appointment_detail_modal {
            frame.render_widget(modal.clone(), area);
            return;
        }
    }

    fn render_workspace(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        mut workspace: crate::ui::components::workspace::PatientWorkspace,
    ) {
        match workspace.active_clinical_menu {
            ClinicalMenuKind::Consultations => {
                if let Some(ref mut clinical) = workspace.clinical {
                    clinical.show_consultations();
                }
                self.render_workspace_clinical(frame, area);
            }
            ClinicalMenuKind::Vitals => {
                if let Some(ref mut clinical) = workspace.clinical {
                    clinical.show_vital_signs();
                }
                self.render_workspace_clinical(frame, area);
            }
            ClinicalMenuKind::Allergies => {
                if let Some(ref mut clinical) = workspace.clinical {
                    clinical.show_allergies();
                }
                self.render_workspace_clinical(frame, area);
            }
            ClinicalMenuKind::MedicalHistory => {
                if let Some(ref mut clinical) = workspace.clinical {
                    clinical.show_medical_history();
                }
                self.render_workspace_clinical(frame, area);
            }
            ClinicalMenuKind::FamilyHistory => {
                if let Some(ref mut clinical) = workspace.clinical {
                    clinical.show_family_history();
                }
                self.render_workspace_clinical(frame, area);
            }
            ClinicalMenuKind::SocialHistory => {
                if let Some(ref mut clinical) = workspace.clinical {
                    clinical.show_social_history();
                }
                self.render_workspace_clinical(frame, area);
            }
        }
    }

    fn render_workspace_summary(&mut self, frame: &mut Frame, area: Rect) {
        use crate::ui::components::workspace::SummaryView;

        if let Some(workspace) = self.workspace_manager.active() {
            let summary = SummaryView::new(workspace.patient_snapshot.clone(), self.theme.clone())
                .with_clinical(workspace.clinical.clone())
                .with_billing(workspace.billing.clone())
                .with_appointments(workspace.appointments.clone());

            frame.render_widget(summary, area);
        }
    }

    fn render_workspace_demographics(&mut self, frame: &mut Frame, area: Rect) {
        use crate::ui::components::workspace::DemographicsViewList;

        if let Some(workspace) = self.workspace_manager.active() {
            let view = DemographicsViewList::new(&workspace.patient_snapshot, &self.theme);
            frame.render_widget(view, area);
        }
    }

    fn render_workspace_clinical(&mut self, frame: &mut Frame, area: Rect) {
        use ratatui::widgets::Paragraph;

        if let Some(workspace) = self.workspace_manager.active() {
            if let Some(ref clinical_state) = workspace.clinical {
                match clinical_state.view {
                    ClinicalView::PatientSummary => {
                        use crate::ui::components::clinical::PatientSummaryComponent;
                        
                        let summary = PatientSummaryComponent::with_patient(
                            Some(workspace.patient_snapshot.clone()),
                            self.theme.clone(),
                        )
                        .with_allergies(clinical_state.allergies.allergies.clone())
                        .with_conditions(clinical_state.medical_history.medical_history.clone())
                        .with_consultations(clinical_state.consultations.consultations.clone())
                        .with_vitals(
                            clinical_state.vitals.vital_signs.first().cloned(),
                        );

                        frame.render_widget(summary, area);
                    }
                    ClinicalView::Consultations => {
                        let mut consultation_list = clinical_state.consultations.consultation_list.clone();
                        consultation_list.consultations =
                            clinical_state.consultations.consultations.clone();

                        frame.render_widget(consultation_list, area);
                    }
                    ClinicalView::ConsultationSummary => {
                        let mut consultation_list = clinical_state.consultations.consultation_list.clone();
                        consultation_list.consultations =
                            clinical_state.consultations.consultations.clone();

                        frame.render_widget(consultation_list, area);
                    }
                    ClinicalView::Allergies => {
                        let mut allergy_list = clinical_state.allergies.allergy_list.clone();
                        allergy_list.allergies = clinical_state.allergies.allergies.clone();

                        frame.render_widget(allergy_list, area);
                    }
                    ClinicalView::VitalSigns => {
                        let mut vitals_list = clinical_state.vitals.vitals_list.clone();
                        vitals_list.vitals = clinical_state.vitals.vital_signs.clone();

                        frame.render_widget(vitals_list, area);
                    }
                    ClinicalView::MedicalHistory => {
                        let mut medical_history_list = clinical_state.medical_history.medical_history_list.clone();
                        medical_history_list.conditions =
                            clinical_state.medical_history.medical_history.clone();

                        frame.render_widget(medical_history_list, area);
                    }
                    ClinicalView::FamilyHistory => {
                        let mut family_history_list = clinical_state.family_history.family_history_list.clone();
                        family_history_list.entries =
                            clinical_state.family_history.family_history.clone();

                        frame.render_widget(family_history_list, area);
                    }
                    ClinicalView::SocialHistory => {
                        use crate::ui::components::clinical::SocialHistoryComponent;
                        
                        let social_history = SocialHistoryComponent::new(
                            self.theme.clone(),
                            &clinical_state.social_history.social_history_config,
                        );
                        // TODO: Populate social_history component with data from clinical_state.social_history.social_history
                        // Requires converting domain SocialHistory to UI SocialHistoryData

                        frame.render_widget(social_history, area);
                    }
                }
            } else {
                frame.render_widget(Paragraph::new("Loading clinical data..."), area);
            }
        }
    }

    fn render_workspace_billing(&mut self, frame: &mut Frame, area: Rect) {
        use crate::ui::components::billing::{BillingView, ClaimList, InvoiceDetail, InvoiceList, PaymentList};
        use ratatui::widgets::Paragraph;

        if let Some(workspace) = self.workspace_manager.active() {
            if let Some(ref billing_state) = workspace.billing {
                match billing_state.view {
                    BillingView::InvoiceList => {
                        let mut invoice_list = InvoiceList::new(self.theme.clone());
                        invoice_list.set_invoices(billing_state.invoices.clone());
                        invoice_list.render(frame, area);
                    }
                    BillingView::InvoiceDetail(invoice_id) => {
                        let mut invoice_detail = InvoiceDetail::new();
                        if let Some(invoice) = billing_state.invoices.iter().find(|i| i.id == invoice_id) {
                            invoice_detail.set_invoice(invoice.clone());
                            invoice_detail.set_payments(
                                billing_state.payments.iter()
                                    .filter(|p| p.invoice_id == invoice_id)
                                    .cloned()
                                    .collect()
                            );
                        }
                        invoice_detail.render(area, frame.buffer_mut());
                    }
                    BillingView::ClaimList => {
                        let claim_list = ClaimList::new(billing_state.claims.clone(), self.theme.clone());
                        claim_list.render(area, frame.buffer_mut());
                    }
                    BillingView::PaymentList => {
                        let payment_list = PaymentList::new(billing_state.payments.clone(), self.theme.clone());
                        payment_list.render(area, frame.buffer_mut());
                    }
                }
            } else {
                frame.render_widget(Paragraph::new("Loading billing data..."), area);
            }
        }
    }

    fn render_workspace_appointments(&mut self, frame: &mut Frame, area: Rect) {
        use crate::ui::components::workspace::PatientAppointmentsView;
        use ratatui::widgets::Paragraph;

        if let Some(workspace) = self.workspace_manager.active() {
            if let Some(ref appt_state) = workspace.appointments {
                if appt_state.appointments.is_empty() && !appt_state.loading {
                    let view = PatientAppointmentsView::new(self.theme.clone());
                    frame.render_widget(view, area);
                } else if appt_state.loading {
                    frame.render_widget(Paragraph::new("Loading appointments..."), area);
                } else {
                    let view = PatientAppointmentsView::new(self.theme.clone());
                    frame.render_widget(view, area);
                }
            } else {
                frame.render_widget(Paragraph::new("Loading appointments..."), area);
            }
        }
    }

    #[cfg(feature = "pathology")]
    fn render_workspace_pathology(&mut self, frame: &mut Frame, area: Rect) {
        use ratatui::widgets::Paragraph;
        let para = Paragraph::new("Pathology subtab — implementation pending");
        frame.render_widget(para, area);
    }

    #[cfg(feature = "prescription")]
    fn render_workspace_prescription(&mut self, frame: &mut Frame, area: Rect) {
        use ratatui::widgets::Paragraph;
        let para = Paragraph::new("Prescription subtab — implementation pending");
        frame.render_widget(para, area);
    }

    #[cfg(feature = "referral")]
    fn render_workspace_referral(&mut self, frame: &mut Frame, area: Rect) {
        use ratatui::widgets::Paragraph;
        let para = Paragraph::new("Referral subtab — implementation pending");
        frame.render_widget(para, area);
    }

    #[cfg(feature = "immunisation")]
    fn render_workspace_immunisation(&mut self, frame: &mut Frame, area: Rect) {
        use ratatui::widgets::Paragraph;
        let para = Paragraph::new("Immunisation subtab — implementation pending");
        frame.render_widget(para, area);
    }

    fn render_welcome_panel(&self, frame: &mut Frame, area: Rect) {
        use crate::ui::components::welcome_panel::WelcomePanel;

        let panel = WelcomePanel::new(self.theme.clone());
        frame.render_widget(panel, area);
    }
}
