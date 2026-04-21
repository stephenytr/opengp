//! Patient Tab Bar Component
//!
//! Colour-coded patient tabs with wrapping.
//! Row 1: Coloured blocks + truncated patient names (wrap if overflow)

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};
use uuid::Uuid;

use crate::ui::theme::Theme;
use crate::ui::shared::{hover_style, selected_hover_style};

/// Patient data for tab display
#[derive(Debug, Clone)]
pub struct PatientTab {
    /// Patient ID
    pub id: Uuid,
    /// Patient full name
    pub name: String,
    /// Colour for this patient's tab
    pub colour: Color,
}

impl PatientTab {
    /// Create a new patient tab
    pub fn new(id: Uuid, name: String, colour: Color) -> Self {
        Self { id, name, colour }
    }

    /// Truncate name to 20 chars with ellipsis if needed
    fn truncated_name(&self) -> String {
        if self.name.len() > 20 {
            format!("{}…", &self.name[..19])
        } else {
            self.name.clone()
        }
    }
}

/// PatientTabBar widget
pub struct PatientTabBar {
    patients: Vec<PatientTab>,
    active_index: Option<usize>,
    theme: Theme,
    hovered_index: Option<usize>,
}

impl PatientTabBar {
    pub fn new(
        patients: Vec<PatientTab>,
        active_index: usize,
        theme: Theme,
    ) -> Self {
        Self {
            patients,
            active_index: Some(active_index),
            theme,
            hovered_index: None,
        }
    }

    pub fn with_no_active(mut self) -> Self {
        self.active_index = None;
        self
    }

    /// Set the hovered tab index for hover styling
    pub fn with_hovered(mut self, hovered_index: Option<usize>) -> Self {
        self.hovered_index = hovered_index;
        self
    }

    /// Get the number of rows needed for patient tabs
    pub fn row_count(&self) -> u16 {
        if self.patients.is_empty() {
            1
        } else {
            self.calculate_patient_rows() as u16
        }
    }

    /// Calculate how many rows of patient tabs are needed
    fn calculate_patient_rows(&self) -> usize {
        if self.patients.is_empty() {
            1
        } else {
            let mut current_row = 0;
            let mut current_width = 0;
            let max_width = 120;

            for patient in &self.patients {
                let tab_width = 3 + patient.truncated_name().len();
                if current_width + tab_width > max_width && current_width > 0 {
                    current_row += 1;
                    current_width = tab_width + 2;
                } else {
                    current_width += tab_width + 2;
                }
            }

            current_row + 1
        }
    }

    pub fn active_patient(&self) -> Option<&PatientTab> {
        self.active_index.and_then(|i| self.patients.get(i))
    }
}

impl Widget for PatientTabBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let patient_rows = self.calculate_patient_rows() as u16;
        let patient_area = Rect::new(
            area.x,
            area.y,
            area.width,
            patient_rows,
        );

        if !self.patients.is_empty() && !patient_area.is_empty() {
            self.render_patient_tabs(patient_area, buf);
        }
    }
}

impl PatientTabBar {
    /// Render patient tabs with wrapping
    fn render_patient_tabs(&self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() || self.patients.is_empty() {
            return;
        }

        let mut x = area.x;
        let mut y = area.y;
        let max_x = area.x + area.width;

        for (idx, patient) in self.patients.iter().enumerate() {
            let is_active = self.active_index == Some(idx);
            let tab_text = format!("■ {}", patient.truncated_name());
            let tab_width = (tab_text.len() + 2) as u16; // +2 for padding

            // Check if tab fits on current row
            if x + tab_width > max_x && x > area.x {
                // Wrap to next row
                x = area.x;
                y += 1;
                if y >= area.y + area.height {
                    break; // Out of space
                }
            }

            // Render the tab
            let is_hovered = self.hovered_index == Some(idx);
            let tab_style = if is_active && is_hovered {
                selected_hover_style(&self.theme)
                    .fg(patient.colour)
            } else if is_active {
                Style::default()
                    .fg(patient.colour)
                    .add_modifier(Modifier::BOLD)
                    .bg(self.theme.colors.selected)
            } else if is_hovered {
                hover_style(&self.theme)
            } else {
                Style::default().fg(patient.colour)
            };

            let tab_rect = Rect::new(x, y, tab_width, 1);
            if !tab_rect.is_empty() {
                let line = Line::from(vec![Span::styled(tab_text, tab_style)]);
                let para = Paragraph::new(line);
                para.render(tab_rect, buf);
            }

            x += tab_width;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_theme() -> Theme {
        Theme::dark()
    }

    #[test]
    fn test_truncate_name_short() {
        let tab = PatientTab::new(Uuid::new_v4(), "John Doe".to_string(), Color::Cyan);
        assert_eq!(tab.truncated_name(), "John Doe");
    }

    #[test]
    fn test_truncate_name_long() {
        let tab = PatientTab::new(
            Uuid::new_v4(),
            "Verylongpatientnamethatexceedstwentychars".to_string(),
            Color::Cyan,
        );
        let truncated = tab.truncated_name();
        assert!(truncated.ends_with('…'));
        assert_eq!(truncated.chars().count(), 20); // 19 chars + ellipsis (grapheme count)
    }

    #[test]
    fn test_single_patient_no_wrap() {
        let patients = vec![PatientTab::new(
            Uuid::new_v4(),
            "John Doe".to_string(),
            Color::Cyan,
        )];
        let theme = create_test_theme();
        let bar = PatientTabBar::new(
            patients,
            0,
            theme,
        );

        assert_eq!(bar.calculate_patient_rows(), 1);
    }

    #[test]
    fn test_multiple_patients_fit_one_row() {
        let mut patients = Vec::new();
        for i in 0..4 {
            patients.push(PatientTab::new(
                Uuid::new_v4(),
                format!("Patient {}", i),
                Color::Cyan,
            ));
        }

        let theme = create_test_theme();
        let bar = PatientTabBar::new(patients, 0, theme);

        assert_eq!(bar.calculate_patient_rows(), 1);
    }

    #[test]
    fn test_many_patients_wrap() {
        let mut patients = Vec::new();
        for i in 0..15 {
            patients.push(PatientTab::new(
                Uuid::new_v4(),
                format!("VeryLongPatientNameNumber{}", i),
                Color::Cyan,
            ));
        }

        let theme = create_test_theme();
        let bar = PatientTabBar::new(patients, 0, theme);

        assert!(bar.calculate_patient_rows() > 1);
    }

    #[test]
    fn test_active_patient() {
        let patients = vec![
            PatientTab::new(Uuid::new_v4(), "Alice".to_string(), Color::Cyan),
            PatientTab::new(Uuid::new_v4(), "Bob".to_string(), Color::Green),
        ];

        let theme = create_test_theme();
        let bar = PatientTabBar::new(patients.clone(), 1, theme);

        let active = bar.active_patient();
        assert!(active.is_some());
        assert_eq!(active.unwrap().name, "Bob");
    }

    #[test]
    fn test_row_count() {
        let patients = vec![PatientTab::new(
            Uuid::new_v4(),
            "John Doe".to_string(),
            Color::Cyan,
        )];

        let theme = create_test_theme();
        let bar = PatientTabBar::new(
            patients,
            0,
            theme,
        );

        assert_eq!(bar.row_count(), 1);
    }

    #[test]
    fn test_empty_patients() {
        let theme = create_test_theme();
        let bar = PatientTabBar::new(vec![], 0, theme);

        assert_eq!(bar.calculate_patient_rows(), 1);
        assert_eq!(bar.row_count(), 1);
    }
}
