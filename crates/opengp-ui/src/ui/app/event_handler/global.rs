use crate::ui::app::App;
use crate::ui::components::appointment::AppointmentView;
use crate::ui::components::clinical_row::{ClinicalMenuKind, ClinicalRow};
use crate::ui::components::status_bar::STATUS_BAR_HEIGHT;
use crate::ui::components::tabs::Tab;
use crossterm::event::MouseEvent;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

impl App {
    pub fn handle_global_mouse_event(&mut self, mouse: MouseEvent, area: Rect) {
        const MAIN_TAB_WIDTH: u16 = 38;
        let main_tab_bar_area = Rect::new(area.x, area.y, MAIN_TAB_WIDTH, 1);
        let patient_tab_bar_area = Rect::new(
            area.x + MAIN_TAB_WIDTH,
            area.y,
            area.width.saturating_sub(MAIN_TAB_WIDTH),
            1,
        );

        if self
            .tab_bar
            .handle_mouse(mouse, main_tab_bar_area)
            .is_some()
        {
            self.refresh_status_bar();
            self.refresh_context();
            return;
        }

        if !self.workspace_manager.workspaces.is_empty() {
            if self
                .workspace_manager
                .handle_patient_tab_mouse(mouse, patient_tab_bar_area)
                .is_some()
            {
                self.tab_bar.select(Tab::PatientWorkspace);
                self.refresh_status_bar();
                self.refresh_context();
                return;
            }

            if let Some(workspace) = self.workspace_manager.active_mut() {
                let clinical_row_area = Rect::new(area.x, area.y + 1, area.width, 1);
                let clinical_items = ClinicalMenuKind::all();
                let active_clinical_idx = workspace.active_clinical_menu.index();
                let mut clinical_row = ClinicalRow::new(
                    clinical_items,
                    active_clinical_idx,
                    workspace.colour,
                    self.theme.clone(),
                );
                if let Some(idx) = clinical_row.handle_mouse(mouse, clinical_row_area) {
                    if let Some(kind) = ClinicalMenuKind::from_index(idx) {
                        workspace.active_clinical_menu = kind;
                        self.refresh_status_bar();
                        return;
                    }
                }
                self.hovered_clinical_menu = clinical_row.hovered_index();
            }
        }

        if let Some(ref mut form) = self.patient_form {
            if let Some(action) = form.handle_mouse(mouse, area) {
                match action {
                    crate::ui::components::patient::PatientFormAction::FocusChanged => {}
                    crate::ui::components::patient::PatientFormAction::ValueChanged => {}
                    crate::ui::components::patient::PatientFormAction::Submit => {}
                    crate::ui::components::patient::PatientFormAction::Cancel => {}
                    crate::ui::components::patient::PatientFormAction::SaveComplete => {
                        self.request_refresh_patients();
                    }
                }
                return;
            }
        }

        let clinical_row_offset = if self.workspace_manager.active().is_some() {
            2
        } else {
            1
        };

        if self.tab_bar.selected() == Tab::PatientSearch && self.patient_form.is_none() {
            let content_area = Rect::new(
                area.x,
                area.y + clinical_row_offset,
                area.width,
                area.height
                    .saturating_sub(clinical_row_offset + STATUS_BAR_HEIGHT),
            );
            if let Some(action) = self.patient_list.handle_mouse(mouse, content_area) {
                match action {
                    crate::ui::components::patient::PatientListAction::SelectionChanged => {}
                    crate::ui::components::patient::PatientListAction::OpenPatient(id) => {
                        self.request_edit_patient(id);
                    }
                    crate::ui::components::patient::PatientListAction::FocusSearch => {}
                    crate::ui::components::patient::PatientListAction::SearchChanged => {}
                    crate::ui::components::patient::PatientListAction::ContextMenu {
                        x: _,
                        y: _,
                        patient_id: _,
                    } => {}
                }
            }
        }

        if self.tab_bar.selected() == Tab::Schedule {
            use crate::ui::components::appointment::schedule::ScheduleAction;

            let appointment_content_area = Rect::new(
                area.x,
                area.y + clinical_row_offset,
                area.width,
                area.height
                    .saturating_sub(clinical_row_offset + STATUS_BAR_HEIGHT),
            );

            match self.appointment_state.current_view {
                AppointmentView::Calendar => {
                    self.appointment_state.calendar.focused = true;
                    self.appointment_state.focused = false;
                    if let Some(action) = self
                        .appointment_state
                        .calendar
                        .handle_mouse(mouse, appointment_content_area)
                    {
                        match action {
                            crate::ui::components::appointment::CalendarAction::SelectDate(
                                date,
                            ) => {
                                self.appointment_state.selected_date = Some(date);
                                self.appointment_state.current_view = AppointmentView::Schedule;
                                self.request_refresh_appointments(date);
                                self.refresh_context();
                            }
                            crate::ui::components::appointment::CalendarAction::FocusDate(_) => {}
                            crate::ui::components::appointment::CalendarAction::MonthChanged(_) => {
                            }
                            crate::ui::components::appointment::CalendarAction::GoToToday => {}
                        }
                    }
                }
                AppointmentView::Schedule => {
                    let chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Length(23), Constraint::Min(0)])
                        .split(appointment_content_area);

                    // Always forward mouse events to calendar for hover tracking
                    if let Some(action) = self
                        .appointment_state
                        .calendar
                        .handle_mouse(mouse, chunks[0])
                    {
                        self.appointment_state.calendar.focused = true;
                        self.appointment_state.focused = false;
                        match action {
                            crate::ui::components::appointment::CalendarAction::SelectDate(
                                date,
                            ) => {
                                self.appointment_state.selected_date = Some(date);
                                self.request_refresh_appointments(date);
                            }
                            crate::ui::components::appointment::CalendarAction::FocusDate(_) => {}
                            crate::ui::components::appointment::CalendarAction::MonthChanged(_) => {
                            }
                            crate::ui::components::appointment::CalendarAction::GoToToday => {}
                        }
                    }

                    if let Some(action) = self.appointment_state.handle_mouse(mouse, chunks[1]) {
                        self.appointment_state.focused = true;
                        self.appointment_state.calendar.focused = false;
                        match action {
                            ScheduleAction::SelectPractitioner(id) => {
                                self.appointment_state.selected_practitioner = Some(id);
                            }
                            ScheduleAction::SelectAppointment(id) => {
                                self.appointment_state.selected_appointment = Some(id);
                            }
                            ScheduleAction::NavigateTimeSlot(_) => {}
                            ScheduleAction::NavigatePractitioner(_) => {}
                            ScheduleAction::ToggleColumn => {}
                            ScheduleAction::CreateAtSlot { .. } => {}
                        }
                    }
                }
            }
        }

        if self.tab_bar.selected() == Tab::PatientWorkspace {
            if let Some(workspace) = self.workspace_manager.active_mut() {
                if let Some(ref mut billing_state) = workspace.billing {
                    use crate::ui::components::billing::{
                        BillingView, ClaimList, InvoiceList, PaymentList,
                    };
                    use crossterm::event::MouseEventKind;

                    let billing_content_area = Rect::new(
                        area.x,
                        area.y + clinical_row_offset,
                        area.width,
                        area.height
                            .saturating_sub(clinical_row_offset + STATUS_BAR_HEIGHT),
                    );

                    if matches!(
                        mouse.kind,
                        MouseEventKind::ScrollUp | MouseEventKind::ScrollDown
                    ) {
                        match billing_state.view {
                            BillingView::ClaimList => {
                                for _ in 0..3 {
                                    if matches!(mouse.kind, MouseEventKind::ScrollUp) {
                                        billing_state.claim_selected_index =
                                            billing_state.claim_selected_index.saturating_sub(1);
                                    } else if billing_state.claim_selected_index
                                        < billing_state.claims.len().saturating_sub(1)
                                    {
                                        billing_state.claim_selected_index += 1;
                                    }
                                }
                                return;
                            }
                            BillingView::InvoiceList => {
                                for _ in 0..3 {
                                    if matches!(mouse.kind, MouseEventKind::ScrollUp) {
                                        billing_state.invoice_selected_index =
                                            billing_state.invoice_selected_index.saturating_sub(1);
                                    } else if billing_state.invoice_selected_index
                                        < billing_state.invoices.len().saturating_sub(1)
                                    {
                                        billing_state.invoice_selected_index += 1;
                                    }
                                }
                                return;
                            }
                            BillingView::PaymentList => {
                                for _ in 0..3 {
                                    if matches!(mouse.kind, MouseEventKind::ScrollUp) {
                                        billing_state.payment_selected_index =
                                            billing_state.payment_selected_index.saturating_sub(1);
                                    } else if billing_state.payment_selected_index
                                        < billing_state.payments.len().saturating_sub(1)
                                    {
                                        billing_state.payment_selected_index += 1;
                                    }
                                }
                                return;
                            }
                            _ => {}
                        }
                    }

                    if matches!(mouse.kind, MouseEventKind::Moved) {
                        use std::time::Duration;
                        if let Some(last) = self.last_billing_render {
                            if last.elapsed() < Duration::from_millis(16) {
                                return;
                            }
                        }
                        self.last_billing_render = Some(std::time::Instant::now());
                    }

                    match billing_state.view {
                        BillingView::ClaimList => {
                            let mut claim_list =
                                ClaimList::new(billing_state.claims.clone(), self.theme.clone());
                            claim_list.selected_index = billing_state.claim_selected_index;
                            if let Some(action) =
                                claim_list.handle_mouse(mouse, billing_content_area)
                            {
                                match action {
                                    crate::ui::components::billing::ClaimListAction::Select(_) => {
                                        billing_state.claim_selected_index = claim_list.selected_index;
                                    }
                                    crate::ui::components::billing::ClaimListAction::ContextMenu { x, y, claim_id } => {
                                        billing_state.claim_selected_index = claim_list.selected_index;
                                        self.show_context_menu(crate::ui::app::ActiveContextMenu::Billing {
                                            billing_type: "claim".into(),
                                            item_id: claim_id,
                                            x,
                                            y,
                                        });
                                    }
                                    _ => {}
                                }
                            }
                        }
                        BillingView::InvoiceList => {
                            let mut invoice_list = InvoiceList::new(self.theme.clone());
                            invoice_list.set_invoices(billing_state.invoices.clone());
                            invoice_list.selected_index = billing_state.invoice_selected_index;
                            if let Some(action) =
                                invoice_list.handle_mouse(mouse, billing_content_area)
                            {
                                match action {
                                    crate::ui::components::billing::InvoiceListAction::Select(_) => {
                                        billing_state.invoice_selected_index = invoice_list.selected_index;
                                    }
                                    crate::ui::components::billing::InvoiceListAction::ContextMenu { x, y, invoice_id } => {
                                        billing_state.invoice_selected_index = invoice_list.selected_index;
                                        self.show_context_menu(crate::ui::app::ActiveContextMenu::Billing {
                                            billing_type: "invoice".into(),
                                            item_id: invoice_id,
                                            x,
                                            y,
                                        });
                                    }
                                    _ => {}
                                }
                            }
                        }
                        BillingView::PaymentList => {
                            let mut payment_list = PaymentList::new(
                                billing_state.payments.clone(),
                                self.theme.clone(),
                            );
                            payment_list.selected_index = billing_state.payment_selected_index;
                            if let Some(action) =
                                payment_list.handle_mouse(mouse, billing_content_area)
                            {
                                match action {
                                    crate::ui::components::billing::PaymentListAction::Select(_) => {
                                        billing_state.payment_selected_index = payment_list.selected_index;
                                    }
                                    crate::ui::components::billing::PaymentListAction::ContextMenu { x, y, payment_id } => {
                                        billing_state.payment_selected_index = payment_list.selected_index;
                                        self.show_context_menu(crate::ui::app::ActiveContextMenu::Billing {
                                            billing_type: "payment".into(),
                                            item_id: payment_id,
                                            x,
                                            y,
                                        });
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if self.tab_bar.selected() == Tab::PatientWorkspace {
            if let Some(workspace) = self.workspace_manager.active_mut() {
                if let Some(ref mut clinical_state) = workspace.clinical {
                    let consultations = clinical_state.consultations.consultations.clone();
                    clinical_state.consultations.consultation_list.consultations = consultations;
                    let allergies = clinical_state.allergies.allergies.clone();
                    clinical_state.allergies.allergy_list.allergies = allergies;
                    let vitals = clinical_state.vitals.vital_signs.clone();
                    clinical_state.vitals.vitals_list.vitals = vitals;
                    let medical_history = clinical_state.medical_history.medical_history.clone();
                    clinical_state
                        .medical_history
                        .medical_history_list
                        .conditions = medical_history;
                    let family_history = clinical_state.family_history.family_history.clone();
                    clinical_state.family_history.family_history_list.entries = family_history;

                    let content_top = area.y + 2;
                    let content_height = area.height.saturating_sub(2 + STATUS_BAR_HEIGHT);
                    let clinical_content_area =
                        Rect::new(area.x, content_top, area.width, content_height);

                    match clinical_state.view {
                        crate::ui::components::clinical::ClinicalView::Consultations => {
                            if let Some(action) = clinical_state
                                .consultations
                                .consultation_list
                                .handle_mouse(mouse, clinical_content_area)
                            {
                                match action {
                                    _ => {}
                                }
                            }
                        }
                        crate::ui::components::clinical::ClinicalView::VitalSigns => {
                            if let Some(action) = clinical_state
                                .vitals
                                .vitals_list
                                .handle_mouse(mouse, clinical_content_area)
                            {
                                match action {
                                    _ => {}
                                }
                            }
                        }
                        crate::ui::components::clinical::ClinicalView::Allergies => {
                            if let Some(action) = clinical_state
                                .allergies
                                .allergy_list
                                .handle_mouse(mouse, clinical_content_area)
                            {
                                match action {
                                    _ => {}
                                }
                            }
                        }
                        crate::ui::components::clinical::ClinicalView::MedicalHistory => {
                            if let Some(action) = clinical_state
                                .medical_history
                                .medical_history_list
                                .handle_mouse(mouse, clinical_content_area)
                            {
                                match action {
                                    _ => {}
                                }
                            }
                        }
                        crate::ui::components::clinical::ClinicalView::FamilyHistory => {
                            if let Some(action) = clinical_state
                                .family_history
                                .family_history_list
                                .handle_mouse(mouse, clinical_content_area)
                            {
                                match action {
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
