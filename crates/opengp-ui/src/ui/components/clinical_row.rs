use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::Widget;
use crossterm::event::{MouseEvent, MouseEventKind, MouseButton};

use crate::ui::theme::Theme;
use crate::ui::shared::hover_style;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClinicalMenuKind {
    Consultations,
    Vitals,
    Allergies,
    MedicalHistory,
    FamilyHistory,
    SocialHistory,
}

impl ClinicalMenuKind {
    pub fn display_name(&self) -> &'static str {
        match self {
            ClinicalMenuKind::Consultations => "Consultations",
            ClinicalMenuKind::Vitals => "Vitals",
            ClinicalMenuKind::Allergies => "Allergies",
            ClinicalMenuKind::MedicalHistory => "Medical History",
            ClinicalMenuKind::FamilyHistory => "Family History",
            ClinicalMenuKind::SocialHistory => "Social History",
        }
    }

    pub fn all() -> Vec<ClinicalMenuKind> {
        vec![
            ClinicalMenuKind::Consultations,
            ClinicalMenuKind::Vitals,
            ClinicalMenuKind::Allergies,
            ClinicalMenuKind::MedicalHistory,
            ClinicalMenuKind::FamilyHistory,
            ClinicalMenuKind::SocialHistory,
        ]
    }

    pub fn index(&self) -> usize {
        match self {
            ClinicalMenuKind::Consultations => 0,
            ClinicalMenuKind::Vitals => 1,
            ClinicalMenuKind::Allergies => 2,
            ClinicalMenuKind::MedicalHistory => 3,
            ClinicalMenuKind::FamilyHistory => 4,
            ClinicalMenuKind::SocialHistory => 5,
        }
    }

    pub fn from_index(index: usize) -> Option<ClinicalMenuKind> {
        match index {
            0 => Some(ClinicalMenuKind::Consultations),
            1 => Some(ClinicalMenuKind::Vitals),
            2 => Some(ClinicalMenuKind::Allergies),
            3 => Some(ClinicalMenuKind::MedicalHistory),
            4 => Some(ClinicalMenuKind::FamilyHistory),
            5 => Some(ClinicalMenuKind::SocialHistory),
            _ => None,
        }
    }

    pub fn next(&self) -> ClinicalMenuKind {
        match self {
            ClinicalMenuKind::Consultations => ClinicalMenuKind::Vitals,
            ClinicalMenuKind::Vitals => ClinicalMenuKind::Allergies,
            ClinicalMenuKind::Allergies => ClinicalMenuKind::MedicalHistory,
            ClinicalMenuKind::MedicalHistory => ClinicalMenuKind::FamilyHistory,
            ClinicalMenuKind::FamilyHistory => ClinicalMenuKind::SocialHistory,
            ClinicalMenuKind::SocialHistory => ClinicalMenuKind::Consultations,
        }
    }

    pub fn prev(&self) -> ClinicalMenuKind {
        match self {
            ClinicalMenuKind::Consultations => ClinicalMenuKind::SocialHistory,
            ClinicalMenuKind::Vitals => ClinicalMenuKind::Consultations,
            ClinicalMenuKind::Allergies => ClinicalMenuKind::Vitals,
            ClinicalMenuKind::MedicalHistory => ClinicalMenuKind::Allergies,
            ClinicalMenuKind::FamilyHistory => ClinicalMenuKind::MedicalHistory,
            ClinicalMenuKind::SocialHistory => ClinicalMenuKind::FamilyHistory,
        }
    }
}

pub struct ClinicalRow {
    items: Vec<ClinicalMenuKind>,
    active_index: usize,
    patient_colour: Color,
    theme: Theme,
    hovered_index: Option<usize>,
}

impl ClinicalRow {
    pub fn new(
        items: Vec<ClinicalMenuKind>,
        active_index: usize,
        patient_colour: Color,
        theme: Theme,
    ) -> Self {
        Self {
            items,
            active_index,
            patient_colour,
            theme,
            hovered_index: None,
        }
    }

    pub fn with_hovered(mut self, hovered_index: Option<usize>) -> Self {
        self.hovered_index = hovered_index;
        self
    }

    pub fn active_item(&self) -> Option<ClinicalMenuKind> {
        self.items.get(self.active_index).copied()
    }

    pub fn move_next(&mut self) {
        if !self.items.is_empty() {
            self.active_index = (self.active_index + 1) % self.items.len();
        }
    }

    pub fn move_prev(&mut self) {
        if !self.items.is_empty() {
            self.active_index = if self.active_index == 0 {
                self.items.len() - 1
            } else {
                self.active_index - 1
            };
        }
    }

    pub fn select_index(&mut self, index: usize) {
        if index < self.items.len() {
            self.active_index = index;
        }
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<usize> {
        if !area.contains(Position::new(mouse.column, mouse.row)) {
            self.hovered_index = None;
            return None;
        }

        let tab_width = (area.width as usize / self.items.len()).max(1);
        let hovered = (mouse.column.saturating_sub(area.x)) as usize / tab_width;

        match mouse.kind {
            MouseEventKind::Moved => {
                if hovered < self.items.len() {
                    self.hovered_index = Some(hovered);
                }
                None
            }
            MouseEventKind::Down(MouseButton::Left) => {
                if hovered < self.items.len() {
                    self.active_index = hovered;
                    self.hovered_index = None;
                    Some(hovered)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl Widget for ClinicalRow {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() || self.items.is_empty() {
            return;
        }

        let tab_width = (area.width as usize / self.items.len()).max(1);

        for (i, item) in self.items.iter().enumerate() {
            let x = area.x + (i * tab_width) as u16;
            let tab_area = Rect::new(x, area.y, tab_width as u16, area.height);

            if tab_area.is_empty() {
                continue;
            }

            let is_active = i == self.active_index;
            let is_hovered = self.hovered_index == Some(i);

            let label = format!(" {} ", item.display_name());

            let style = if is_active {
                Style::default()
                    .bg(self.patient_colour)
                    .fg(self.theme.colors.background)
            } else if is_hovered {
                hover_style(&self.theme)
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
    fn test_clinical_menu_kind_display_names() {
        assert_eq!(ClinicalMenuKind::Consultations.display_name(), "Consultations");
        assert_eq!(ClinicalMenuKind::Vitals.display_name(), "Vitals");
        assert_eq!(ClinicalMenuKind::Allergies.display_name(), "Allergies");
        assert_eq!(ClinicalMenuKind::MedicalHistory.display_name(), "Medical History");
        assert_eq!(ClinicalMenuKind::FamilyHistory.display_name(), "Family History");
        assert_eq!(ClinicalMenuKind::SocialHistory.display_name(), "Social History");
    }

    #[test]
    fn test_clinical_menu_kind_all_count() {
        let all = ClinicalMenuKind::all();
        assert_eq!(all.len(), 6);
    }

    #[test]
    fn test_clinical_menu_kind_next_wraps() {
        assert_eq!(ClinicalMenuKind::Consultations.next(), ClinicalMenuKind::Vitals);
        assert_eq!(ClinicalMenuKind::SocialHistory.next(), ClinicalMenuKind::Consultations);
    }

    #[test]
    fn test_clinical_menu_kind_prev_wraps() {
        assert_eq!(ClinicalMenuKind::Vitals.prev(), ClinicalMenuKind::Consultations);
        assert_eq!(ClinicalMenuKind::Consultations.prev(), ClinicalMenuKind::SocialHistory);
    }

    #[test]
    fn test_clinical_row_move_next() {
        let items = ClinicalMenuKind::all();
        let mut row = ClinicalRow::new(items, 0, Color::Blue, Theme::dark());
        
        row.move_next();
        assert_eq!(row.active_item(), Some(ClinicalMenuKind::Vitals));
        
        row.move_next();
        assert_eq!(row.active_item(), Some(ClinicalMenuKind::Allergies));
    }

    #[test]
    fn test_clinical_row_move_prev() {
        let items = ClinicalMenuKind::all();
        let mut row = ClinicalRow::new(items, 2, Color::Blue, Theme::dark());
        
        row.move_prev();
        assert_eq!(row.active_item(), Some(ClinicalMenuKind::Vitals));
        
        row.move_prev();
        assert_eq!(row.active_item(), Some(ClinicalMenuKind::Consultations));
    }

    #[test]
    fn test_clinical_row_wrap_around() {
        let items = ClinicalMenuKind::all();
        let mut row = ClinicalRow::new(items, 5, Color::Blue, Theme::dark());
        
        row.move_next();
        assert_eq!(row.active_item(), Some(ClinicalMenuKind::Consultations));
        
        row.move_prev();
        assert_eq!(row.active_item(), Some(ClinicalMenuKind::SocialHistory));
    }

    #[test]
    fn test_clinical_row_render() {
        let items = ClinicalMenuKind::all();
        let row = ClinicalRow::new(items, 0, Color::Blue, Theme::dark());

        let mut buf = Buffer::empty(Rect::new(0, 0, 100, 1));
        row.render(Rect::new(0, 0, 100, 1), &mut buf);

        assert!(!buf.content.is_empty());
    }
}