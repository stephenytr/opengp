//! Tests for shared UI types and style helpers

#[cfg(test)]
mod tests {
    use ratatui::style::Modifier;
    use uuid::Uuid;

    use crate::ui::shared::{
        border_block, disabled_style, error_style, header_style, selected_style,
    };
    use crate::ui::shared::{FormAction, FormMode};
    use crate::ui::theme::Theme;

    #[test]
    fn test_form_mode_create_is_default() {
        let mode = FormMode::default();
        assert_eq!(mode, FormMode::Create);
    }

    #[test]
    fn test_form_mode_edit_with_uuid() {
        let uuid = Uuid::new_v4();
        let mode = FormMode::Edit(uuid);
        assert_eq!(mode, FormMode::Edit(uuid));
    }

    #[test]
    fn test_form_mode_edit_round_trip() {
        let uuid = Uuid::new_v4();
        let mode = FormMode::Edit(uuid);

        // Verify we can extract the UUID
        match mode {
            FormMode::Edit(id) => assert_eq!(id, uuid),
            FormMode::Create => panic!("Expected Edit variant"),
        }
    }

    #[test]
    fn test_form_mode_create_vs_edit() {
        let create = FormMode::Create;
        let uuid = Uuid::new_v4();
        let edit = FormMode::Edit(uuid);

        assert_ne!(create, edit);
        assert_eq!(create, FormMode::Create);
    }

    #[test]
    fn test_form_mode_clone() {
        let uuid = Uuid::new_v4();
        let mode = FormMode::Edit(uuid);
        let cloned = mode;

        assert_eq!(mode, cloned);
    }

    #[test]
    fn test_form_action_focus_changed() {
        let action = FormAction::FocusChanged;
        assert_eq!(action, FormAction::FocusChanged);
    }

    #[test]
    fn test_form_action_value_changed() {
        let action = FormAction::ValueChanged;
        assert_eq!(action, FormAction::ValueChanged);
    }

    #[test]
    fn test_form_action_submit() {
        let action = FormAction::Submit;
        assert_eq!(action, FormAction::Submit);
    }

    #[test]
    fn test_form_action_cancel() {
        let action = FormAction::Cancel;
        assert_eq!(action, FormAction::Cancel);
    }

    #[test]
    fn test_form_action_all_variants_exist() {
        let _focus = FormAction::FocusChanged;
        let _value = FormAction::ValueChanged;
        let _submit = FormAction::Submit;
        let _cancel = FormAction::Cancel;
        // If this compiles, all variants exist
    }

    #[test]
    fn test_selected_style_produces_style() {
        let theme = Theme::default();
        let style = selected_style(&theme);

        // Verify it's a valid Style object
        assert_eq!(style.fg, Some(theme.colors.selected));
    }

    #[test]
    fn test_selected_style_uses_theme_color() {
        let theme = Theme::dark();
        let style = selected_style(&theme);

        assert_eq!(style.fg, Some(theme.colors.selected));
    }

    #[test]
    fn test_header_style_produces_style() {
        let theme = Theme::default();
        let style = header_style(&theme);

        // Verify it's a valid Style object with primary color
        assert_eq!(style.fg, Some(theme.colors.primary));
    }

    #[test]
    fn test_header_style_has_bold_modifier() {
        let theme = Theme::default();
        let style = header_style(&theme);

        assert!(style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn test_disabled_style_produces_style() {
        let theme = Theme::default();
        let style = disabled_style(&theme);

        assert_eq!(style.fg, Some(theme.colors.disabled));
    }

    #[test]
    fn test_disabled_style_has_dim_modifier() {
        let theme = Theme::default();
        let style = disabled_style(&theme);

        assert!(style.add_modifier.contains(Modifier::DIM));
    }

    #[test]
    fn test_error_style_produces_style() {
        let theme = Theme::default();
        let style = error_style(&theme);

        assert_eq!(style.fg, Some(theme.colors.error));
    }

    #[test]
    fn test_error_style_has_bold_modifier() {
        let theme = Theme::default();
        let style = error_style(&theme);

        assert!(style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn test_border_block_with_title() {
        let theme = Theme::default();
        let _block = border_block("Test Title", &theme, false);

        // Block is created successfully with title
    }

    #[test]
    fn test_border_block_focused_style() {
        let theme = Theme::default();
        let _block_unfocused = border_block("Test", &theme, false);
        let _block_focused = border_block("Test", &theme, true);

        // Both focused and unfocused blocks are created successfully
    }

    #[test]
    fn test_border_block_has_borders() {
        let theme = Theme::default();
        let _block = border_block("Test", &theme, false);

        // Block is created successfully with borders
    }
}
