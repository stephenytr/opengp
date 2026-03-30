use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Widget};

use super::{AllergyForm, AllergyFormField, DropdownWidget};
use crate::ui::layout::LABEL_WIDTH;
use crate::ui::widgets::TextareaWidget;

impl Widget for AllergyForm {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let block = Block::default()
            .title(" New Allergy ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.form_state.theme.colors.border));

        block.clone().render(area, buf);

        let inner = block.inner(area);
        if inner.is_empty() {
            return;
        }

        let label_width = LABEL_WIDTH;
        let field_start = inner.x + label_width + 2;

        let fields = AllergyFormField::all();

        let mut total_height: u16 = 0;
        for field in &fields {
            if field.is_textarea() {
                total_height += self
                    .textarea_for(*field)
                    .map(|state| state.height())
                    .unwrap_or(2);
            } else if field.is_dropdown() {
                total_height += 4;
            } else {
                total_height += 2;
            }
        }
        self.form_state.scroll.set_total_height(total_height);
        self.form_state
            .scroll
            .clamp_offset(inner.height.saturating_sub(2));

        let mut y: i32 = (inner.y as i32) + 1 - (self.form_state.scroll.scroll_offset as i32);
        let max_y = inner.y as i32 + inner.height as i32 - 2;

        let mut open_dropdown: Option<(DropdownWidget, Rect)> = None;

        for field in fields {
            let field_height = if field.is_textarea() {
                self.textarea_for(field)
                    .map(|state| state.height())
                    .unwrap_or(2) as i32
            } else if field.is_dropdown() {
                4
            } else {
                2
            };

            if y + field_height <= inner.y as i32 || y >= max_y {
                y += field_height;
                continue;
            }

            let is_focused = field == self.form_state.focused_field;

            if field.is_textarea() {
                let Some(textarea_state) = self.textarea_for(field) else {
                    y += field_height;
                    continue;
                };

                let textarea_height = textarea_state.height();
                if y >= inner.y as i32 && y < max_y {
                    let field_area =
                        Rect::new(inner.x + 1, y as u16, inner.width - 2, textarea_height);
                    TextareaWidget::new(textarea_state, self.form_state.theme.clone())
                        .focused(is_focused)
                        .render(field_area, buf);

                    if let Some(error_msg) = self.error(field) {
                        if (y as u16) + textarea_height <= inner.y + inner.height - 2 {
                            let error_style =
                                Style::default().fg(self.form_state.theme.colors.error);
                            buf.set_string(
                                inner.x + 1,
                                (y as u16) + textarea_height,
                                format!("  {}", error_msg),
                                error_style,
                            );
                        }
                    }
                }
                y += textarea_height as i32;
                continue;
            }

            let has_error = self.error(field).is_some();

            if y >= inner.y as i32 && y < max_y && !field.is_dropdown() {
                let label_style = if is_focused {
                    Style::default()
                        .fg(self.form_state.theme.colors.primary)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(self.form_state.theme.colors.foreground)
                };

                buf.set_string(inner.x + 1, y as u16, field.label(), label_style);

                if is_focused {
                    buf.set_string(
                        field_start - 1,
                        y as u16,
                        ">",
                        Style::default().fg(self.form_state.theme.colors.primary),
                    );
                }
            }

            let max_value_width = inner.width.saturating_sub(label_width + 4);

            match field {
                AllergyFormField::AllergyType | AllergyFormField::Severity => {
                    let Some(dropdown) = self.dropdown_for(field).cloned() else {
                        y += 4;
                        continue;
                    };

                    if y >= inner.y as i32 && y < max_y {
                        let dropdown_area = Rect::new(field_start, y as u16, max_value_width, 3);
                        if dropdown.is_open() {
                            open_dropdown = Some((dropdown.clone(), dropdown_area));
                        }
                        dropdown.focused(is_focused).render(dropdown_area, buf);
                        if let Some(error_msg) = self.error(field) {
                            let error_style =
                                Style::default().fg(self.form_state.theme.colors.error);
                            buf.set_string(
                                field_start,
                                (y as u16) + 3,
                                format!("  {}", error_msg),
                                error_style,
                            );
                        }
                    }
                    y += 4;
                }
                AllergyFormField::OnsetDate => {
                    if y >= inner.y as i32 && y < max_y {
                        let value = self.get_value(field);
                        let value_style = if has_error {
                            Style::default().fg(self.form_state.theme.colors.error)
                        } else {
                            Style::default().fg(self.form_state.theme.colors.foreground)
                        };

                        let display_value = if value.len() > max_value_width as usize {
                            &value[value.len() - max_value_width as usize..]
                        } else {
                            &value
                        };

                        buf.set_string(field_start, y as u16, display_value, value_style);

                        if let Some(error_msg) = self.error(field) {
                            let error_style =
                                Style::default().fg(self.form_state.theme.colors.error);
                            buf.set_string(
                                field_start,
                                (y as u16) + 1,
                                format!("  {}", error_msg),
                                error_style,
                            );
                        }
                    }
                    y += 2;
                }
                _ => {
                    y += 2;
                }
            }
        }

        if let Some((dropdown, dropdown_area)) = open_dropdown {
            dropdown.render(dropdown_area, buf);
        }

        self.form_state.scroll.render_scrollbar(inner, buf);

        let help_y = inner.y + inner.height - 1;
        buf.set_string(
            inner.x + 1,
            help_y,
            "Tab: Next | Ctrl+S: Submit | Esc: Cancel",
            Style::default().fg(self.form_state.theme.colors.disabled),
        );

        if self.date_picker.is_visible() {
            self.date_picker.render(area, buf);
        }
    }
}
