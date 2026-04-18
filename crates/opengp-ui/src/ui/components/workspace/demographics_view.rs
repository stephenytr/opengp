//! Demographics View Component
//!
//! Read-only panel displaying patient demographic information in a two-column layout.
//! Displays all demographic fields without truncation.

use chrono::Datelike;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

use crate::ui::layout::LABEL_WIDTH;
use crate::ui::shared::header_style;
use crate::ui::theme::Theme;
use crate::ui::view_models::PatientFormData;

/// Read-only demographics view widget.
pub struct DemographicsView<'a> {
    pub data: &'a PatientFormData,
    pub theme: &'a Theme,
}

impl<'a> DemographicsView<'a> {
    pub fn new(data: &'a PatientFormData, theme: &'a Theme) -> Self {
        Self { data, theme }
    }

    /// Calculate age from date of birth.
    fn calculate_age(&self) -> u32 {
        let today = chrono::Local::now().naive_local().date();
        let dob = self.data.date_of_birth;
        let mut age = today.year() - dob.year();
        if today.month() < dob.month() || (today.month() == dob.month() && today.day() < dob.day()) {
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
        let display_value = value
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("Not provided");
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

        if let Some(title) = &self.data.title {
            lines.push(self.format_field("Title", title));
        }

        lines.push(self.format_field("First Name", &self.data.first_name));

        if let Some(middle) = &self.data.middle_name {
            lines.push(self.format_field("Middle Name", middle));
        }

        lines.push(self.format_field("Last Name", &self.data.last_name));

        if let Some(preferred) = &self.data.preferred_name {
            lines.push(self.format_field("Preferred Name", preferred));
        }

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

        if let Some(expiry) = self.data.medicare_expiry {
            lines.push(self.format_field(
                "Medicare Expiry",
                &expiry.format("%d/%m/%Y").to_string(),
            ));
        } else {
            lines.push(self.format_field("Medicare Expiry", "Not provided"));
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

        // Address
        if let Some(line1) = &self.data.address_line1 {
            lines.push(self.format_field("Address Line 1", line1));
        } else {
            lines.push(self.format_field("Address Line 1", "Not provided"));
        }

        if let Some(line2) = &self.data.address_line2 {
            lines.push(self.format_field("Address Line 2", line2));
        }

        if let Some(suburb) = &self.data.suburb {
            lines.push(self.format_field("Suburb", suburb));
        } else {
            lines.push(self.format_field("Suburb", "Not provided"));
        }

        if let Some(state) = &self.data.state {
            lines.push(self.format_field("State", state));
        } else {
            lines.push(self.format_field("State", "Not provided"));
        }

        if let Some(postcode) = &self.data.postcode {
            lines.push(self.format_field("Postcode", postcode));
        } else {
            lines.push(self.format_field("Postcode", "Not provided"));
        }

        if let Some(country) = &self.data.country {
            lines.push(self.format_field("Country", country));
        } else {
            lines.push(self.format_field("Country", "Not provided"));
        }

        // Phone and email
        lines.push(self.format_optional_field("Phone (Home)", &self.data.phone_home));
        lines.push(self.format_optional_field("Phone (Mobile)", &self.data.phone_mobile));
        lines.push(self.format_optional_field("Email", &self.data.email));

        lines.push(Line::from(""));

        // Emergency contact section
        lines.push(Line::from(Span::styled(
            "EMERGENCY CONTACT",
            header_style(self.theme),
        )));

        lines.push(self.format_optional_field(
            "Name",
            &self.data.emergency_contact_name,
        ));
        lines.push(self.format_optional_field(
            "Phone",
            &self.data.emergency_contact_phone,
        ));
        lines.push(self.format_optional_field(
            "Relationship",
            &self.data.emergency_contact_relationship,
        ));

        lines.push(Line::from(""));

        // Additional info section
        lines.push(Line::from(Span::styled(
            "ADDITIONAL INFORMATION",
            header_style(self.theme),
        )));

        if let Some(concession_type) = &self.data.concession_type {
            lines.push(self.format_field("Concession Type", &concession_type.to_string()));
        } else {
            lines.push(self.format_field("Concession Type", "Not provided"));
        }

        lines.push(self.format_optional_field(
            "Concession Number",
            &self.data.concession_number,
        ));

        if let Some(atsi) = &self.data.aboriginal_torres_strait_islander {
            lines.push(self.format_field("ATSI Status", &atsi.to_string()));
        } else {
            lines.push(self.format_field("ATSI Status", "Not provided"));
        }

        lines.push(self.format_optional_field(
            "Preferred Language",
            &self.data.preferred_language,
        ));

        let interpreter_str = if self.data.interpreter_required {
            "Yes"
        } else {
            "No"
        };
        lines.push(self.format_field("Interpreter Required", interpreter_str));

        lines
    }
}

impl<'a> Widget for DemographicsView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < 3 {
            return;
        }

        // Create border block
        let block = Block::default()
            .title("Demographics")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.width < 2 || inner.height < 2 {
            return;
        }

        // Split into two columns
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(inner);

        // Build column content
        let left_lines = self.build_left_column();
        let right_lines = self.build_right_column();

        // Render left column
        let left_para = Paragraph::new(left_lines);
        left_para.render(chunks[0], buf);

        // Render right column
        let right_para = Paragraph::new(right_lines);
        right_para.render(chunks[1], buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use opengp_domain::domain::patient::{AtsiStatus, ConcessionType, Gender};

    fn create_test_data() -> PatientFormData {
        PatientFormData {
            title: Some("Dr".to_string()),
            first_name: "John".to_string(),
            middle_name: Some("Michael".to_string()),
            last_name: "Smith".to_string(),
            preferred_name: Some("Johnny".to_string()),
            date_of_birth: NaiveDate::from_ymd_opt(1990, 5, 15).unwrap(),
            gender: Gender::Male,
            ihi: Some("8003608166690176".to_string()),
            medicare_number: Some("29435261061".to_string()),
            medicare_irn: Some(1),
            medicare_expiry: Some(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
            address_line1: Some("123 Main Street".to_string()),
            address_line2: Some("Apt 4B".to_string()),
            suburb: Some("Sydney".to_string()),
            state: Some("NSW".to_string()),
            postcode: Some("2000".to_string()),
            country: Some("Australia".to_string()),
            phone_home: Some("(02) 9555 1234".to_string()),
            phone_mobile: Some("0412 345 678".to_string()),
            email: Some("john.smith@example.com".to_string()),
            emergency_contact_name: Some("Jane Smith".to_string()),
            emergency_contact_phone: Some("0412 345 679".to_string()),
            emergency_contact_relationship: Some("Spouse".to_string()),
            concession_type: Some(ConcessionType::Pensioner),
            concession_number: Some("123456789".to_string()),
            preferred_language: Some("English".to_string()),
            interpreter_required: false,
            aboriginal_torres_strait_islander: Some(AtsiStatus::NeitherAboriginalNorTorresStrait),
        }
    }

    #[test]
    fn test_demographics_view_new() {
        let data = create_test_data();
        let theme = Theme::default();
        let view = DemographicsView::new(&data, &theme);

        assert_eq!(view.data.first_name, "John");
    }

    #[test]
    fn test_calculate_age() {
        let data = create_test_data();
        let theme = Theme::default();
        let view = DemographicsView::new(&data, &theme);

        let age = view.calculate_age();
        // Age should be approximately 34 (2026 - 1990 - 1 since birthday hasn't passed yet in April)
        assert!(age >= 33 && age <= 36, "Age calculation seems off: {}", age);
    }

    #[test]
    fn test_build_left_column_has_sections() {
        let data = create_test_data();
        let theme = Theme::default();
        let view = DemographicsView::new(&data, &theme);

        let lines = view.build_left_column();
        let text: String = lines.iter().map(|l| l.to_string()).collect();

        assert!(text.contains("IDENTIFICATION"));
        assert!(text.contains("HEALTHCARE IDENTIFIERS"));
        assert!(text.contains("John"));
        assert!(text.contains("Smith"));
    }

    #[test]
    fn test_build_right_column_has_sections() {
        let data = create_test_data();
        let theme = Theme::default();
        let view = DemographicsView::new(&data, &theme);

        let lines = view.build_right_column();
        let text: String = lines.iter().map(|l| l.to_string()).collect();

        assert!(text.contains("CONTACT DETAILS"));
        assert!(text.contains("EMERGENCY CONTACT"));
        assert!(text.contains("ADDITIONAL INFORMATION"));
        assert!(text.contains("Sydney"));
    }

    #[test]
    fn test_format_optional_field_with_value() {
        let data = create_test_data();
        let theme = Theme::default();
        let view = DemographicsView::new(&data, &theme);

        let line = view.format_optional_field("Email", &data.email);
        let text = line.to_string();

        assert!(text.contains("john.smith@example.com"));
    }

    #[test]
    fn test_format_optional_field_without_value() {
        let mut data = create_test_data();
        data.email = None;
        let theme = Theme::default();
        let view = DemographicsView::new(&data, &theme);

        let line = view.format_optional_field("Email", &data.email);
        let text = line.to_string();

        assert!(text.contains("Not provided"));
    }
}
