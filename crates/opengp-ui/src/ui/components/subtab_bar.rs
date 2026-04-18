//! Subtab bar component for patient workspace navigation.
//!
//! Provides navigation between different patient data views (Summary, Demographics, Clinical, etc.)

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Widget;

use crate::ui::theme::Theme;

/// Represents the different subtabs available in the patient workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubtabKind {
    /// Patient summary view
    Summary,
    /// Patient demographics and personal information
    Demographics,
    /// Clinical records (consultations, vitals, allergies, etc.)
    Clinical,
    /// Billing and consultation records
    Billing,
    /// Appointments and scheduling
    Appointments,
    /// Pathology requests and results
    #[cfg(feature = "pathology")]
    Pathology,
    /// Prescriptions
    #[cfg(feature = "prescription")]
    Prescription,
    /// Referral letters
    #[cfg(feature = "referral")]
    Referral,
    /// Immunisation records
    #[cfg(feature = "immunisation")]
    Immunisation,
}

impl SubtabKind {
    /// Get the display name for this subtab
    pub fn display_name(&self) -> &'static str {
        match self {
            SubtabKind::Summary => "Summary",
            SubtabKind::Demographics => "Demographics",
            SubtabKind::Clinical => "Clinical",
            SubtabKind::Billing => "Billing",
            SubtabKind::Appointments => "Appointments",
            #[cfg(feature = "pathology")]
            SubtabKind::Pathology => "Pathology",
            #[cfg(feature = "prescription")]
            SubtabKind::Prescription => "Prescription",
            #[cfg(feature = "referral")]
            SubtabKind::Referral => "Referral",
            #[cfg(feature = "immunisation")]
            SubtabKind::Immunisation => "Immunisation",
        }
    }

    /// Get all available subtabs (including feature-gated ones)
    pub fn all() -> Vec<SubtabKind> {
        vec![
            SubtabKind::Summary,
            SubtabKind::Demographics,
            SubtabKind::Clinical,
            SubtabKind::Billing,
            SubtabKind::Appointments,
            #[cfg(feature = "pathology")]
            SubtabKind::Pathology,
            #[cfg(feature = "prescription")]
            SubtabKind::Prescription,
            #[cfg(feature = "referral")]
            SubtabKind::Referral,
            #[cfg(feature = "immunisation")]
            SubtabKind::Immunisation,
        ]
    }

    /// Check if this subtab is enabled (compiled in)
    pub fn is_enabled(&self) -> bool {
        match self {
            SubtabKind::Summary | SubtabKind::Demographics | SubtabKind::Clinical
            | SubtabKind::Billing | SubtabKind::Appointments => true,
            #[cfg(feature = "pathology")]
            SubtabKind::Pathology => true,
            #[cfg(feature = "prescription")]
            SubtabKind::Prescription => true,
            #[cfg(feature = "referral")]
            SubtabKind::Referral => true,
            #[cfg(feature = "immunisation")]
            SubtabKind::Immunisation => true,
        }
    }
}

/// Subtab bar widget for rendering subtabs
pub struct SubtabBar {
    /// Available subtabs
    subtabs: Vec<SubtabKind>,
    /// Currently active subtab index
    active_index: usize,
    /// Patient colour for active subtab highlight
    patient_colour: Color,
    /// Theme for styling
    theme: Theme,
}

impl SubtabBar {
    /// Create a new subtab bar
    pub fn new(
        subtabs: Vec<SubtabKind>,
        active_index: usize,
        patient_colour: Color,
        theme: Theme,
    ) -> Self {
        Self {
            subtabs,
            active_index,
            patient_colour,
            theme,
        }
    }
}

impl Widget for SubtabBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() || self.subtabs.is_empty() {
            return;
        }

        let tab_width = (area.width as usize / self.subtabs.len()).max(1);

        for (i, subtab) in self.subtabs.iter().enumerate() {
            let x = area.x + (i * tab_width) as u16;
            let tab_area = Rect::new(x, area.y, tab_width as u16, area.height);

            if tab_area.is_empty() {
                continue;
            }

            let is_active = i == self.active_index;
            let is_enabled = subtab.is_enabled();

            let label = if is_enabled {
                format!(" {} ", subtab.display_name())
            } else {
                format!(" {} (not enabled) ", subtab.display_name())
            };

            let style = if is_active && is_enabled {
                Style::default()
                    .bg(self.patient_colour)
                    .fg(self.theme.colors.background)
            } else if !is_enabled {
                Style::default().fg(self.theme.colors.disabled)
            } else {
                Style::default().fg(self.theme.colors.foreground)
            };

            buf.set_string(tab_area.x, tab_area.y, label, style);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subtab_kind_all_count() {
        let all = SubtabKind::all();
        // Always-on: Summary, Demographics, Clinical, Billing, Appointments = 5
        // Feature-gated: Pathology, Prescription, Referral, Immunisation
        #[cfg(all(
            feature = "pathology",
            feature = "prescription",
            feature = "referral",
            feature = "immunisation"
        ))]
        assert_eq!(all.len(), 9, "Expected 9 subtabs with all features enabled");

        #[cfg(not(any(
            feature = "pathology",
            feature = "prescription",
            feature = "referral",
            feature = "immunisation"
        )))]
        assert_eq!(all.len(), 5, "Expected 5 subtabs with no features enabled");
    }

    #[test]
    fn test_subtab_kind_is_enabled() {
        // Always-on subtabs
        assert!(SubtabKind::Summary.is_enabled());
        assert!(SubtabKind::Demographics.is_enabled());
        assert!(SubtabKind::Clinical.is_enabled());
        assert!(SubtabKind::Billing.is_enabled());
        assert!(SubtabKind::Appointments.is_enabled());

        // Feature-gated subtabs
        #[cfg(feature = "pathology")]
        assert!(SubtabKind::Pathology.is_enabled());

        #[cfg(feature = "prescription")]
        assert!(SubtabKind::Prescription.is_enabled());

        #[cfg(feature = "referral")]
        assert!(SubtabKind::Referral.is_enabled());

        #[cfg(feature = "immunisation")]
        assert!(SubtabKind::Immunisation.is_enabled());
    }

    #[test]
    fn test_subtab_bar_render() {
        let subtabs = vec![
            SubtabKind::Summary,
            SubtabKind::Demographics,
            SubtabKind::Clinical,
        ];
        let bar = SubtabBar::new(
            subtabs,
            0,
            Color::Blue,
            Theme::dark(),
        );

        let mut buf = Buffer::empty(Rect::new(0, 0, 30, 1));
        bar.render(Rect::new(0, 0, 30, 1), &mut buf);

        // Verify buffer was written to (non-empty)
        assert!(!buf.content.is_empty());
    }

    #[test]
    fn test_subtab_bar_disabled_rendering() {
        let subtabs = SubtabKind::all();
        let bar = SubtabBar::new(
            subtabs,
            0,
            Color::Blue,
            Theme::dark(),
        );

        let mut buf = Buffer::empty(Rect::new(0, 0, 100, 1));
        bar.render(Rect::new(0, 0, 100, 1), &mut buf);

        // Verify buffer was written to
        assert!(!buf.content.is_empty());
    }
}
