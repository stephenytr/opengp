//! Demographics View Component (List-based)
//!
//! Read-only panel displaying patient demographic information from PatientListItem.
//! Used in workspace context where only list-view fields are available.

use chrono::Datelike;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

use crate::ui::layout::LABEL_WIDTH;
use crate::ui::shared::header_style;
use crate::ui::theme::Theme;
use crate::ui::view_models::PatientListItem;

/// Read-only demographics view widget using PatientListItem data.
#[derive(Clone)]
pub struct DemographicsViewList<'a> {
    pub data: &'a PatientListItem,
    pub theme: &'a Theme,
}

impl<'a> DemographicsViewList<'a> {
    /// Create a new demographics view from PatientListItem.
    pub fn new(data: &'a PatientListItem, theme: &'a Theme) -> Self {
        Self { data, theme }
    }

    /// Calculate age from date of birth.
    fn calculate_age(&self) -> u32 {
        let today = chrono::Local::now().naive_local().date();
        let dob = self.data.date_of_birth;
        let mut age = today.year() - dob.year();
        if today.month() < dob.month()
            || (today.month() == dob.month() && today.day() < dob.day())
        {
            age -= 1;
        }
        age as u32
    }

    /// Format a field label and value as a line.
    fn format_field(&self, label: &str, value: &str) -> Line<'static> {
        let label_span = Span::styled(
            format!("{:<width$}", label, width = LABEL_WIDTH as usize),
            header_style(self.theme),
        );
        let value_span = Span::raw(value.to_string());
        Line::from(vec![label_span, value_span])
    }

    /// Format an optional field, showing "Not provided" if None.
    fn format_optional_field(&self, label: &str, value: &Option<String>) -> Line<'static> {
        let display_value = value.as_ref().map(|s| s.as_str()).unwrap_or("Not provided");
        self.format_field(label, display_value)
    }

    /// Build left column lines (identification and healthcare identifiers).
    fn build_left_column(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        // Identification section
        lines.push(Line::from(Span::styled(
            "IDENTIFICATION",
            header_style(self.theme),
        )));

        lines.push(self.format_field("Full Name", &self.data.full_name));

        lines.push(self.format_field(
            "Date of Birth",
            &self.data.date_of_birth.format("%d/%m/%Y").to_string(),
        ));

        let age = self.calculate_age();
        lines.push(self.format_field("Age", &age.to_string()));

        lines.push(self.format_field("Gender", &self.data.gender.to_string()));

        lines.push(Line::from(""));

        // Healthcare identifiers section
        lines.push(Line::from(Span::styled(
            "HEALTHCARE IDENTIFIERS",
            header_style(self.theme),
        )));

        lines.push(self.format_optional_field("IHI", &self.data.ihi));

        lines.push(self.format_optional_field("Medicare Number", &self.data.medicare_number));

        if let Some(irn) = self.data.medicare_irn {
            lines.push(self.format_field("Medicare IRN", &irn.to_string()));
        } else {
            lines.push(self.format_field("Medicare IRN", "Not provided"));
        }

        lines
    }

    /// Build right column lines (contact details and additional info).
    fn build_right_column(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        // Contact details section
        lines.push(Line::from(Span::styled(
            "CONTACT DETAILS",
            header_style(self.theme),
        )));

        lines.push(self.format_optional_field("Mobile Phone", &self.data.phone_mobile));

        lines.push(self.format_field("Address", "See Demographics tab for full address"));

        lines.push(Line::from(""));

        // Additional info section
        lines.push(Line::from(Span::styled(
            "ADDITIONAL INFORMATION",
            header_style(self.theme),
        )));

        lines.push(self.format_field(
            "Note",
            "Full demographics available in patient record",
        ));

        lines
    }
}

impl<'a> Widget for DemographicsViewList<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < 3 {
            return;
        }

        let block = Block::default()
            .title(" Demographics ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.width < 2 || inner.height < 2 {
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(inner);

        let left_lines = self.build_left_column();
        let right_lines = self.build_right_column();

        let left_para = Paragraph::new(left_lines);
        left_para.render(chunks[0], buf);

        let right_para = Paragraph::new(right_lines);
        right_para.render(chunks[1], buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use opengp_domain::domain::patient::Gender;
    use uuid::Uuid;

    fn create_test_patient_list_item() -> PatientListItem {
        PatientListItem {
            id: Uuid::new_v4(),
            full_name: "Dr John Michael Smith".to_string(),
            date_of_birth: NaiveDate::from_ymd_opt(1990, 5, 15).unwrap(),
            gender: Gender::Male,
            medicare_number: Some("29435261061".to_string()),
            medicare_irn: Some(1),
            ihi: Some("8003608166690176".to_string()),
            phone_mobile: Some("0412 345 678".to_string()),
        }
    }

    #[test]
    fn test_demographics_view_list_new() {
        let data = create_test_patient_list_item();
        let theme = Theme::default();
        let view = DemographicsViewList::new(&data, &theme);
        assert_eq!(view.data.full_name, "Dr John Michael Smith");
    }

    #[test]
    fn test_calculate_age() {
        let data = create_test_patient_list_item();
        let theme = Theme::default();
        let view = DemographicsViewList::new(&data, &theme);
        let age = view.calculate_age();
        // Age should be approximately 35 (2026 - 1990 - 1 since birthday hasn't passed yet in April)
        assert!(age >= 33 && age <= 37, "Age calculation seems off: {}", age);
    }

    #[test]
    fn test_build_left_column_has_sections() {
        let data = create_test_patient_list_item();
        let theme = Theme::default();
        let view = DemographicsViewList::new(&data, &theme);
        let lines = view.build_left_column();
        let text: String = lines.iter().map(|l| l.to_string()).collect();
        assert!(text.contains("IDENTIFICATION"));
        assert!(text.contains("HEALTHCARE IDENTIFIERS"));
        assert!(text.contains("Dr John Michael Smith"));
    }

    #[test]
    fn test_build_right_column_has_sections() {
        let data = create_test_patient_list_item();
        let theme = Theme::default();
        let view = DemographicsViewList::new(&data, &theme);
        let lines = view.build_right_column();
        let text: String = lines.iter().map(|l| l.to_string()).collect();
        assert!(text.contains("CONTACT DETAILS"));
        assert!(text.contains("ADDITIONAL INFORMATION"));
    }

    #[test]
    fn test_format_optional_field_with_value() {
        let data = create_test_patient_list_item();
        let theme = Theme::default();
        let view = DemographicsViewList::new(&data, &theme);
        let line = view.format_optional_field("Medicare Number", &data.medicare_number);
        let text = line.to_string();
        assert!(text.contains("29435261061"));
    }

    #[test]
    fn test_format_optional_field_without_value() {
        let mut data = create_test_patient_list_item();
        data.medicare_number = None;
        let theme = Theme::default();
        let view = DemographicsViewList::new(&data, &theme);
        let line = view.format_optional_field("Medicare Number", &data.medicare_number);
        let text = line.to_string();
        assert!(text.contains("Not provided"));
    }

    #[test]
    fn test_render_does_not_panic() {
        let data = create_test_patient_list_item();
        let theme = Theme::default();
        let view = DemographicsViewList::new(&data, &theme);
        let area = Rect::new(0, 0, 80, 20);
        let mut buf = Buffer::empty(area);
        view.render(area, &mut buf);
        assert!(!buf.content.is_empty());
    }
}
