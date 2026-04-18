//! Patient Tab Bar Component
//!
//! Two-tier, colour-coded patient tabs with wrapping and subtab bar.
//! Row 1: Coloured blocks + truncated patient names (wrap if overflow)
//! Row 2: SubtabBar for active patient

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};
use uuid::Uuid;

use crate::ui::components::subtab_bar::{SubtabBar, SubtabKind};
use crate::ui::theme::Theme;

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
    /// Open patient tabs
    patients: Vec<PatientTab>,
    /// Index of active patient
    active_index: usize,
    /// Subtabs to show for active patient
    subtabs: Vec<SubtabKind>,
    /// Active subtab index
    active_subtab_index: usize,
    /// Theme for styling
    theme: Theme,
}

impl PatientTabBar {
    /// Create a new patient tab bar
    pub fn new(
        patients: Vec<PatientTab>,
        active_index: usize,
        subtabs: Vec<SubtabKind>,
        active_subtab_index: usize,
        theme: Theme,
    ) -> Self {
        Self {
            patients,
            active_index,
            subtabs,
            active_subtab_index,
            theme,
        }
    }

    /// Get the number of rows needed (patient tabs + 1 for subtab bar)
    pub fn row_count(&self) -> u16 {
        if self.patients.is_empty() {
            1
        } else {
            // Calculate rows needed for patient tabs + 1 for subtab bar
            let patient_row_count = self.calculate_patient_rows() as u16;
            patient_row_count + 1
        }
    }

    /// Calculate how many rows of patient tabs are needed
    fn calculate_patient_rows(&self) -> usize {
        if self.patients.is_empty() {
            0
        } else {
            // Each tab is "■ name" (1 block + 1 space + name, min 4 chars)
            // Simple heuristic: max 120 chars per row, ~4-5 tabs typically fit
            let mut current_row = 0;
            let mut current_width = 0;
            let max_width = 120;

            for patient in &self.patients {
                let tab_width = 3 + patient.truncated_name().len(); // "■ " + name
                if current_width + tab_width > max_width && current_width > 0 {
                    current_row += 1;
                    current_width = tab_width + 2; // +2 for spacing
                } else {
                    current_width += tab_width + 2; // +2 for spacing between tabs
                }
            }

            current_row + 1
        }
    }

    /// Get the active patient
    pub fn active_patient(&self) -> Option<&PatientTab> {
        self.patients.get(self.active_index)
    }
}

impl Widget for PatientTabBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        // Split into patient tabs area and subtab bar area
        let patient_rows = self.calculate_patient_rows() as u16;
        let vertical_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(patient_rows),
                Constraint::Min(1),
            ])
            .split(area);

        let patient_area = vertical_split[0];
        let subtab_area = vertical_split[1];

        // Render patient tabs
        if !self.patients.is_empty() && !patient_area.is_empty() {
            self.render_patient_tabs(patient_area, buf);
        }

        // Render subtab bar
        if !self.subtabs.is_empty() && !subtab_area.is_empty() {
            if let Some(patient) = self.active_patient() {
                let subtab_bar = SubtabBar::new(
                    self.subtabs.clone(),
                    self.active_subtab_index,
                    patient.colour,
                    self.theme.clone(),
                );
                subtab_bar.render(subtab_area, buf);
            }
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
            let is_active = idx == self.active_index;
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
            let tab_style = if is_active {
                Style::default()
                    .fg(patient.colour)
                    .add_modifier(Modifier::BOLD)
                    .bg(self.theme.colors.selected)
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
            vec![SubtabKind::Summary],
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
        let bar = PatientTabBar::new(patients, 0, vec![SubtabKind::Summary], 0, theme);

        // 4 short names should fit in one row (max 120 chars)
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
        let bar = PatientTabBar::new(patients, 0, vec![SubtabKind::Summary], 0, theme);

        // 15 long names should require > 1 row
        assert!(bar.calculate_patient_rows() > 1);
    }

    #[test]
    fn test_active_patient() {
        let patients = vec![
            PatientTab::new(Uuid::new_v4(), "Alice".to_string(), Color::Cyan),
            PatientTab::new(Uuid::new_v4(), "Bob".to_string(), Color::Green),
        ];

        let theme = create_test_theme();
        let bar = PatientTabBar::new(patients.clone(), 1, vec![SubtabKind::Summary], 0, theme);

        let active = bar.active_patient();
        assert!(active.is_some());
        assert_eq!(active.unwrap().name, "Bob");
    }

    #[test]
    fn test_row_count_includes_subtab_bar() {
        let patients = vec![PatientTab::new(
            Uuid::new_v4(),
            "John Doe".to_string(),
            Color::Cyan,
        )];

        let theme = create_test_theme();
        let bar = PatientTabBar::new(
            patients,
            0,
            vec![SubtabKind::Summary, SubtabKind::Demographics],
            0,
            theme,
        );

        // 1 patient row + 1 subtab bar row = 2 total
        assert_eq!(bar.row_count(), 2);
    }

    #[test]
    fn test_empty_patients() {
        let theme = create_test_theme();
        let bar = PatientTabBar::new(vec![], 0, vec![], 0, theme);

        assert_eq!(bar.calculate_patient_rows(), 0);
        assert_eq!(bar.row_count(), 1);
    }
}
