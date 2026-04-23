use crate::ui::app::App;
use crate::ui::components::appointment::AppointmentView;
use crate::ui::components::clinical::ClinicalView;
use crate::ui::components::clinical_row::{ClinicalMenuKind, ClinicalRow};
use crate::ui::components::clinical::consultation_list::ConsultationList;
use crate::ui::components::clinical::allergy_list::AllergyList;
use crate::ui::components::clinical::vitals_list::VitalSignsList;
use crate::ui::components::clinical::medical_history_list::MedicalHistoryList;
use crate::ui::components::clinical::family_history_list::FamilyHistoryList;
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
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(STATUS_BAR_HEIGHT),
            ])
            .split(terminal);

        let tab_bar_area = main_layout[0];
        let clinical_row_area = main_layout[1];
        let content_area = main_layout[2];
        let status_bar_area = main_layout[3];

        // Split tab_bar_area horizontally:
        // Left: Main TabBar (Schedule | Patient Search) - fixed ~25 chars
        // Right: PatientTabBar (open patient colour tabs) - fills remaining space
        const MAIN_TAB_WIDTH: u16 = 38;
        let [main_tab_area, patient_tab_area] = Layout::horizontal([
            Constraint::Length(MAIN_TAB_WIDTH),
            Constraint::Min(0),
        ]).areas(tab_bar_area);

        // Render main TabBar on left portion
        self.tab_bar.clone().render(main_tab_area, frame.buffer_mut());

        let patient_workspace_active = self.tab_bar.selected() == Tab::PatientWorkspace
            && self.workspace_manager.active().is_some();

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
            let hovered_patient = self.workspace_manager.hovered_tab.element_id;

            let theme = self.theme.clone();
            let patient_tab_bar = if patient_workspace_active {
                PatientTabBar::new(patient_tabs, active_idx, theme).with_hovered(hovered_patient)
            } else {
                PatientTabBar::new(patient_tabs, active_idx, theme).with_no_active().with_hovered(hovered_patient)
            };
            patient_tab_bar.render(patient_tab_area, frame.buffer_mut());
        }

        if patient_workspace_active {
            let active_workspace = self.workspace_manager.active().unwrap();
            let patient_colour = active_workspace.colour;
            let clinical_items = ClinicalMenuKind::all();
            let active_clinical_idx = active_workspace.active_clinical_menu.index();
            let clinical_row = ClinicalRow::new(
                clinical_items,
                active_clinical_idx,
                patient_colour,
                self.theme.clone(),
            ).with_hovered(self.hovered_clinical_menu);
            clinical_row.render(clinical_row_area, frame.buffer_mut());
        }

        self.render_content(frame, content_area);

        self.render_server_unavailable_banner(frame, content_area);

        // Render context menu if visible
        if let Some(ref ctx_menu) = self.context_menu_state {
            ctx_menu.render(terminal, frame.buffer_mut());
        }

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
            Tab::Schedule => self.render_appointment_tab(frame, area),
            Tab::PatientSearch => self.render_patient_tab(frame, area),
            Tab::PatientWorkspace => {
                self.sync_clinical_view_to_menu();
                self.render_workspace_clinical(frame, area);
            }
        }
    }

    fn sync_clinical_view_to_menu(&mut self) {
        if let Some(workspace) = self.workspace_manager.active_mut() {
            let menu = workspace.active_clinical_menu;
            if let Some(ref mut clinical) = workspace.clinical {
                match menu {
                    ClinicalMenuKind::Consultations => {
                        match clinical.view {
                            ClinicalView::PatientSummary
                            | ClinicalView::Consultations
                            | ClinicalView::ConsultationSummary => {}
                            _ => clinical.show_patient_summary(),
                        }
                    }
                    ClinicalMenuKind::Vitals => {
                        if clinical.view != ClinicalView::VitalSigns {
                            clinical.show_vital_signs();
                        }
                    }
                    ClinicalMenuKind::Allergies => {
                        if clinical.view != ClinicalView::Allergies {
                            clinical.show_allergies();
                        }
                    }
                    ClinicalMenuKind::MedicalHistory => {
                        if clinical.view != ClinicalView::MedicalHistory {
                            clinical.show_medical_history();
                        }
                    }
                    ClinicalMenuKind::FamilyHistory => {
                        if clinical.view != ClinicalView::FamilyHistory {
                            clinical.show_family_history();
                        }
                    }
                    ClinicalMenuKind::SocialHistory => {
                        if clinical.view != ClinicalView::SocialHistory {
                            clinical.show_social_history();
                        }
                    }
                }
            }
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
                    .constraints([Constraint::Length(23), Constraint::Min(0)])
                    .split(area);

                frame.render_widget(self.appointment_state.calendar.clone(), chunks[0]);

                if self.appointment_state.is_loading() {
                    use ratatui::widgets::{Block, Borders};

                    let mut loading_state = self.appointment_state.loading_state.clone();
                    loading_state.tick();
                    let indicator = loading_state.to_indicator(&self.theme);

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



    fn render_workspace_summary(&mut self, frame: &mut Frame, area: Rect) {
        use crate::ui::components::workspace::SummaryView;

        if let Some(workspace) = self.workspace_manager.active() {
            let summary = SummaryView::new(
                &workspace.patient_snapshot,
                &workspace.clinical,
                &workspace.billing,
                &workspace.appointments,
                &self.theme,
            );

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
                        let existing = &clinical_state.consultations.consultation_list;
                        let mut consultation_list = ConsultationList::new(existing.theme.clone());
                        consultation_list.selected_index = existing.selected_index;
                        consultation_list.scroll_offset = existing.scroll_offset;
                        consultation_list.consultations = clinical_state.consultations.consultations.clone();

                        frame.render_widget(consultation_list, area);
                    }
                    ClinicalView::ConsultationSummary => {
                        let existing = &clinical_state.consultations.consultation_list;
                        let mut consultation_list = ConsultationList::new(existing.theme.clone());
                        consultation_list.selected_index = existing.selected_index;
                        consultation_list.scroll_offset = existing.scroll_offset;
                        consultation_list.consultations = clinical_state.consultations.consultations.clone();

                        frame.render_widget(consultation_list, area);
                    }
                    ClinicalView::Allergies => {
                        let existing = &clinical_state.allergies.allergy_list;
                        let mut allergy_list = AllergyList::new(existing.theme.clone());
                        allergy_list.selected_index = existing.selected_index;
                        allergy_list.scroll_offset = existing.scroll_offset;
                        allergy_list.allergies = clinical_state.allergies.allergies.clone();

                        frame.render_widget(allergy_list, area);
                    }
                    ClinicalView::VitalSigns => {
                        let existing = &clinical_state.vitals.vitals_list;
                        let mut vitals_list = VitalSignsList::new(existing.theme.clone());
                        vitals_list.selected_index = existing.selected_index;
                        vitals_list.scroll_offset = existing.scroll_offset;
                        vitals_list.vitals = clinical_state.vitals.vital_signs.clone();

                        frame.render_widget(vitals_list, area);
                    }
                    ClinicalView::MedicalHistory => {
                        let existing = &clinical_state.medical_history.medical_history_list;
                        let mut medical_history_list = MedicalHistoryList::new(existing.theme.clone());
                        medical_history_list.selected_index = existing.selected_index;
                        medical_history_list.scroll_offset = existing.scroll_offset;
                        medical_history_list.conditions = clinical_state.medical_history.medical_history.clone();
                        frame.render_widget(medical_history_list, area);
                    }
                    ClinicalView::FamilyHistory => {
                        let existing = &clinical_state.family_history.family_history_list;
                        let mut family_history_list = FamilyHistoryList::new(existing.theme.clone());
                        family_history_list.selected_index = existing.selected_index;
                        family_history_list.scroll_offset = existing.scroll_offset;
                        family_history_list.entries = clinical_state.family_history.family_history.clone();
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
                let theme = self.theme.clone();
                if appt_state.appointments.is_empty() && !appt_state.loading {
                    let view = PatientAppointmentsView::new(theme.clone());
                    frame.render_widget(view, area);
                } else if appt_state.loading {
                    frame.render_widget(Paragraph::new("Loading appointments..."), area);
                } else {
                    let view = PatientAppointmentsView::new(theme);
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
