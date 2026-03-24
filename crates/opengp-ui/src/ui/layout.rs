//! Layout Constants Module

pub const HEADER_HEIGHT: u16 = 2;
pub const LABEL_WIDTH: u16 = 22;
pub const STATUS_BAR_HEIGHT: u16 = 1;
pub const TIME_COLUMN_WIDTH: u16 = 7;

pub const PATIENT_COL_NAME: u16 = 25;
pub const PATIENT_COL_DOB: u16 = 10;
pub const PATIENT_COL_MEDICARE: u16 = 15;
pub const PATIENT_COL_PHONE: u16 = 15;
pub const PATIENT_COL_LAST_VISIT: u16 = 12;

pub const INNER_PADDING: u16 = 1;
pub const OUTER_MARGIN: u16 = 1;
pub const MIN_FIELD_WIDTH: u16 = 10;
pub const BORDER_WIDTH: u16 = 1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_constants_exist_and_have_expected_values() {
        assert_eq!(HEADER_HEIGHT, 2);
        assert_eq!(LABEL_WIDTH, 22);
        assert_eq!(STATUS_BAR_HEIGHT, 1);
        assert_eq!(TIME_COLUMN_WIDTH, 7);
    }

    #[test]
    fn test_patient_column_widths_are_defined() {
        assert_eq!(PATIENT_COL_NAME, 25);
        assert_eq!(PATIENT_COL_DOB, 10);
        assert_eq!(PATIENT_COL_MEDICARE, 15);
        assert_eq!(PATIENT_COL_PHONE, 15);
        assert_eq!(PATIENT_COL_LAST_VISIT, 12);
    }

    #[test]
    fn test_spacing_and_sizing_constants() {
        assert_eq!(INNER_PADDING, 1);
        assert_eq!(OUTER_MARGIN, 1);
        assert_eq!(MIN_FIELD_WIDTH, 10);
        assert_eq!(BORDER_WIDTH, 1);
    }
}
