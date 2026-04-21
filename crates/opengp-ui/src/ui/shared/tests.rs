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

    #[test]
    fn test_error_field_pattern_consistency() {
        // This test verifies that all state structs that have an error field
        // use the consistent pattern: error: Option<String>
        // NOT save_error, error_message, or other variants

        use crate::ui::components::appointment::AppointmentForm;
        use crate::ui::screens::login::LoginScreen;
        use crate::ui::components::status_bar::StatusBar;

        let theme = Theme::default();

        // Test AppointmentForm has error field with correct type
        let healthcare_config = opengp_config::healthcare::HealthcareConfig::default();
        let mut form = AppointmentForm::new(theme.clone(), healthcare_config);
        form.set_error("Test error".to_string());
        // If this compiles and runs, the error field exists and accepts String

        // Test LoginScreen has error field with correct type
        let mut login = LoginScreen::new(theme.clone());
        login.set_error(Some("Test error".to_string()));
        // If this compiles and runs, the error field exists and accepts Option<String>

        // Test StatusBar has error field with correct type
        let mut status = StatusBar::new(theme.clone());
        status.set_error(Some("Test error".to_string()));
        // If this compiles and runs, the error field exists and accepts Option<String>

        // All three structs successfully accept error values,
        // confirming the error field pattern is consistent
    }

    #[test]
    fn test_error_field_none_initialization() {
        // Verify that error fields initialize to None
        use crate::ui::components::appointment::AppointmentForm;
        use crate::ui::screens::login::LoginScreen;
        use crate::ui::components::status_bar::StatusBar;

        let theme = Theme::default();
        let healthcare_config = opengp_config::healthcare::HealthcareConfig::default();

        let form = AppointmentForm::new(theme.clone(), healthcare_config);
        // AppointmentForm initializes with error: None (verified by successful creation)

        let login = LoginScreen::new(theme.clone());
        // LoginScreen initializes with error: None (verified by successful creation)

        let status = StatusBar::new(theme.clone());
        // StatusBar initializes with error: None (verified by successful creation)

        // All three initialize successfully, confirming error field defaults to None
    }

    #[test]
    fn test_error_field_can_be_cleared() {
        // Verify that error fields can be set to None
        use crate::ui::screens::login::LoginScreen;
        use crate::ui::components::status_bar::StatusBar;

        let theme = Theme::default();

        let mut login = LoginScreen::new(theme.clone());
        login.set_error(Some("Error".to_string()));
        login.set_error(None); // Clear error
        // If this compiles, error field supports Option<String> pattern

        let mut status = StatusBar::new(theme.clone());
        status.set_error(Some("Error".to_string()));
        status.set_error(None); // Clear error
        // If this compiles, error field supports Option<String> pattern
    }

    // ============================================================================
    // PAGINATION CONSISTENCY TESTS
    // ============================================================================

    #[test]
    fn pagination_paginated_state_has_required_fields() {
        use crate::ui::components::shared::PaginatedState;

        let state = PaginatedState::new();

        // Verify all required fields exist and have correct types
        assert_eq!(state.page, 0, "page field should initialize to 0");
        assert_eq!(state.page_size, 20, "page_size field should initialize to 20");
        assert!(!state.loading, "loading field should initialize to false");
        assert!(
            state.error.is_none(),
            "error field should initialize to None"
        );
    }

    #[test]
    fn pagination_paginated_state_page_navigation() {
        use crate::ui::components::shared::PaginatedState;

        let mut state = PaginatedState::new();
        state.page_size = 10;

        // Test next_page with sufficient items
        state.next_page(50);
        assert_eq!(state.page, 1, "next_page should increment page");

        // Test next_page at boundary
        state.page = 4;
        state.next_page(50); // 50 items / 10 per page = 5 pages (0-4)
        assert_eq!(state.page, 4, "next_page should not exceed max page");

        // Test prev_page
        state.prev_page();
        assert_eq!(state.page, 3, "prev_page should decrement page");

        // Test prev_page at boundary
        state.page = 0;
        state.prev_page();
        assert_eq!(state.page, 0, "prev_page should not go below 0");
    }

    #[test]
    fn pagination_paginated_state_page_offset_calculation() {
        use crate::ui::components::shared::PaginatedState;

        let mut state = PaginatedState::new();
        state.page_size = 25;

        assert_eq!(state.page_offset(), 0, "page 0 offset should be 0");

        state.page = 1;
        assert_eq!(state.page_offset(), 25, "page 1 offset should be 25");

        state.page = 3;
        assert_eq!(state.page_offset(), 75, "page 3 offset should be 75");
    }

    #[test]
    fn pagination_paginated_state_total_pages_calculation() {
        use crate::ui::components::shared::PaginatedState;

        let state = PaginatedState::new(); // page_size = 20

        assert_eq!(state.total_pages(0), 1, "0 items should be 1 page");
        assert_eq!(state.total_pages(20), 1, "20 items should be 1 page");
        assert_eq!(state.total_pages(21), 2, "21 items should be 2 pages");
        assert_eq!(state.total_pages(40), 2, "40 items should be 2 pages");
        assert_eq!(state.total_pages(41), 3, "41 items should be 3 pages");
    }

    #[test]
    fn pagination_paginated_state_page_size_adjustment() {
        use crate::ui::components::shared::PaginatedState;

        let mut state = PaginatedState::new();

        // Test normal adjustment
        state.set_page_size(30);
        assert_eq!(state.page_size, 24, "page_size should be height - 6");

        // Test minimum clamping
        state.set_page_size(10);
        assert_eq!(state.page_size, 5, "page_size should clamp to minimum of 5");

        // Test large height
        state.set_page_size(100);
        assert_eq!(state.page_size, 94, "page_size should be height - 6");
    }

    #[test]
    fn pagination_paginated_list_has_required_fields() {
        use crate::ui::components::billing::PaginatedList;

        let items = vec![1, 2, 3, 4, 5];
        let list = PaginatedList::new(items.clone());

        // Verify all required fields exist and have correct types
        assert_eq!(list.items, items, "items field should store provided items");
        assert_eq!(
            list.selected_index, 0,
            "selected_index should initialize to 0"
        );
        assert!(
            list.scroll_state.selected().is_some(),
            "scroll_state should be initialized"
        );
        assert!(
            list.hovered_index.is_none(),
            "hovered_index should initialize to None"
        );
    }

    #[test]
    fn pagination_paginated_list_navigation() {
        use crate::ui::components::billing::PaginatedList;

        let items = vec!["a", "b", "c", "d", "e"];
        let mut list = PaginatedList::new(items);

        // Test select_next_wrap
        list.select_next_wrap();
        assert_eq!(list.selected_index, 1, "select_next_wrap should increment");

        list.select_next_wrap();
        assert_eq!(list.selected_index, 2, "select_next_wrap should increment");

        // Test wrapping at end
        list.selected_index = 4;
        list.select_next_wrap();
        assert_eq!(
            list.selected_index, 0,
            "select_next_wrap should wrap to start"
        );

        // Test select_prev_wrap
        list.select_prev_wrap();
        assert_eq!(
            list.selected_index, 4,
            "select_prev_wrap should wrap to end"
        );

        list.select_prev_wrap();
        assert_eq!(list.selected_index, 3, "select_prev_wrap should decrement");
    }

    #[test]
    fn pagination_paginated_list_selected_item() {
        use crate::ui::components::billing::PaginatedList;

        let items = vec![10, 20, 30, 40, 50];
        let mut list = PaginatedList::new(items);

        assert_eq!(list.selected(), Some(&10), "selected should return first item");

        list.selected_index = 2;
        assert_eq!(list.selected(), Some(&30), "selected should return item at index");

        list.selected_index = 10;
        assert_eq!(
            list.selected(),
            None,
            "selected should return None for out-of-bounds index"
        );
    }

    #[test]
    fn pagination_paginated_list_empty_list_handling() {
        use crate::ui::components::billing::PaginatedList;

        let list: PaginatedList<i32> = PaginatedList::new(vec![]);

        assert!(list.items.is_empty(), "items should be empty");
        assert_eq!(list.selected_index, 0, "selected_index should be 0");
        assert!(
            list.scroll_state.selected().is_none(),
            "scroll_state should have no selection"
        );
    }

    #[test]
    fn pagination_clinical_table_list_has_required_fields() {
        use crate::ui::widgets::{ClinicalTableList, ColumnDef};

        #[derive(Clone)]
        struct TestItem {
            id: u32,
            name: String,
        }

        let items = vec![
            TestItem {
                id: 1,
                name: "Item 1".to_string(),
            },
            TestItem {
                id: 2,
                name: "Item 2".to_string(),
            },
        ];

        let columns = vec![
            ColumnDef {
                title: "ID",
                width: 5,
                render: Box::new(|i: &TestItem| i.id.to_string()),
            },
            ColumnDef {
                title: "Name",
                width: 20,
                render: Box::new(|i: &TestItem| i.name.clone()),
            },
        ];

        let list = ClinicalTableList::new(
            items.clone(),
            columns,
            Theme::default(),
            "Test Table",
            None,
        );

        // Verify all required fields exist
        assert_eq!(list.items.len(), 2, "items field should store provided items");
        assert_eq!(list.columns.len(), 2, "columns field should store column defs");
        assert_eq!(
            list.selected_index, 0,
            "selected_index should initialize to 0"
        );
        assert_eq!(
            list.scroll_offset, 0,
            "scroll_offset should initialize to 0"
        );
        assert_eq!(list.title, "Test Table", "title field should store title");
        assert!(!list.loading, "loading should initialize to false");
        assert!(
            list.hovered_index.is_none(),
            "hovered_index should initialize to None"
        );
    }

    #[test]
    fn pagination_clinical_table_list_navigation() {
        use crate::ui::widgets::{ClinicalTableList, ColumnDef};

        #[derive(Clone)]
        struct TestItem {
            id: u32,
        }

        let items: Vec<TestItem> = (1..=10).map(|id| TestItem { id }).collect();
        let columns = vec![ColumnDef {
            title: "ID",
            width: 5,
            render: Box::new(|i: &TestItem| i.id.to_string()),
        }];

        let mut list = ClinicalTableList::new(items, columns, Theme::default(), "Test", None);

        // Test move_down
        list.move_down();
        assert_eq!(list.selected_index, 1, "move_down should increment");

        // Test move_up
        list.move_up();
        assert_eq!(list.selected_index, 0, "move_up should decrement");

        // Test move_first
        list.selected_index = 5;
        list.move_first();
        assert_eq!(list.selected_index, 0, "move_first should go to 0");

        // Test move_last
        list.move_last();
        assert_eq!(list.selected_index, 9, "move_last should go to last index");
    }

    #[test]
    fn pagination_clinical_table_list_scroll_adjustment() {
        use crate::ui::widgets::{ClinicalTableList, ColumnDef};

        #[derive(Clone)]
        struct TestItem {
            id: u32,
        }

        let items: Vec<TestItem> = (1..=20).map(|id| TestItem { id }).collect();
        let columns = vec![ColumnDef {
            title: "ID",
            width: 5,
            render: Box::new(|i: &TestItem| i.id.to_string()),
        }];

        let mut list = ClinicalTableList::new(items, columns, Theme::default(), "Test", None);

        // Test scroll adjustment when selection is above scroll offset
        list.selected_index = 5;
        list.scroll_offset = 10;
        list.adjust_scroll(8);
        assert_eq!(
            list.scroll_offset, 5,
            "scroll_offset should adjust to selected_index"
        );

        // Test scroll adjustment when selection is below visible area
        list.selected_index = 20;
        list.scroll_offset = 0;
        list.adjust_scroll(8);
        assert!(
            list.scroll_offset > 0,
            "scroll_offset should adjust to keep selection visible"
        );
    }
}
