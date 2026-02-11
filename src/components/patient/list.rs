use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};
use ratatui::Frame;

use crate::components::{Action, Component};
use crate::domain::patient::Patient;
use crate::error::Result;


pub struct PatientListComponent {
    all_patients: Vec<Patient>,
    filtered_patients: Vec<Patient>,
    table_state: TableState,
    #[allow(dead_code)]
    is_loading: bool,
    error_message: Option<String>,
    search_query: String,
    search_mode: bool,
    #[allow(dead_code)]
    page: usize,
    #[allow(dead_code)]
    page_size: usize,
}

impl PatientListComponent {
    pub fn new() -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));
        
        Self {
            all_patients: Vec::new(),
            filtered_patients: Vec::new(),
            table_state,
            is_loading: false,
            error_message: None,
            search_query: String::new(),
            search_mode: false,
            page: 0,
            page_size: 20,
        }
    }

    pub fn with_mock_data() -> Self {
        let mut component = Self::new();
        let mock_patients = Self::generate_mock_patients();
        component.all_patients = mock_patients.clone();
        component.filtered_patients = mock_patients;
        component
    }

    fn generate_mock_patients() -> Vec<Patient> {
        use chrono::NaiveDate;
        use uuid::Uuid;
        use crate::domain::patient::{Address, Gender};

        vec![
            Patient {
                id: Uuid::new_v4(),
                ihi: Some("8003608166690503".to_string()),
                medicare_number: Some("2123456781".to_string()),
                medicare_irn: Some(1),
                medicare_expiry: Some(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
                title: Some("Mr".to_string()),
                first_name: "John".to_string(),
                middle_name: Some("David".to_string()),
                last_name: "Smith".to_string(),
                preferred_name: None,
                date_of_birth: NaiveDate::from_ymd_opt(1980, 5, 15).unwrap(),
                gender: Gender::Male,
                address: Address::default(),
                phone_home: Some("02 9876 5432".to_string()),
                phone_mobile: Some("0412 345 678".to_string()),
                email: Some("john.smith@example.com".to_string()),
                emergency_contact: None,
                concession_type: None,
                concession_number: None,
                preferred_language: "English".to_string(),
                interpreter_required: false,
                aboriginal_torres_strait_islander: None,
                is_active: true,
                is_deceased: false,
                deceased_date: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            Patient {
                id: Uuid::new_v4(),
                ihi: Some("8003608166690511".to_string()),
                medicare_number: Some("3234567892".to_string()),
                medicare_irn: Some(1),
                medicare_expiry: Some(NaiveDate::from_ymd_opt(2026, 6, 30).unwrap()),
                title: Some("Ms".to_string()),
                first_name: "Sarah".to_string(),
                middle_name: None,
                last_name: "Johnson".to_string(),
                preferred_name: Some("Sally".to_string()),
                date_of_birth: NaiveDate::from_ymd_opt(1992, 8, 22).unwrap(),
                gender: Gender::Female,
                address: Address::default(),
                phone_home: None,
                phone_mobile: Some("0423 456 789".to_string()),
                email: Some("sarah.j@example.com".to_string()),
                emergency_contact: None,
                concession_type: None,
                concession_number: None,
                preferred_language: "English".to_string(),
                interpreter_required: false,
                aboriginal_torres_strait_islander: None,
                is_active: true,
                is_deceased: false,
                deceased_date: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            Patient {
                id: Uuid::new_v4(),
                ihi: Some("8003608166690529".to_string()),
                medicare_number: Some("4345678903".to_string()),
                medicare_irn: Some(2),
                medicare_expiry: Some(NaiveDate::from_ymd_opt(2025, 3, 31).unwrap()),
                title: Some("Dr".to_string()),
                first_name: "Michael".to_string(),
                middle_name: Some("James".to_string()),
                last_name: "Chen".to_string(),
                preferred_name: None,
                date_of_birth: NaiveDate::from_ymd_opt(1975, 12, 10).unwrap(),
                gender: Gender::Male,
                address: Address::default(),
                phone_home: Some("03 8765 4321".to_string()),
                phone_mobile: Some("0434 567 890".to_string()),
                email: Some("m.chen@example.com".to_string()),
                emergency_contact: None,
                concession_type: None,
                concession_number: None,
                preferred_language: "English".to_string(),
                interpreter_required: false,
                aboriginal_torres_strait_islander: None,
                is_active: true,
                is_deceased: false,
                deceased_date: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            Patient {
                id: Uuid::new_v4(),
                ihi: Some("8003608166690537".to_string()),
                medicare_number: Some("5456789014".to_string()),
                medicare_irn: Some(1),
                medicare_expiry: Some(NaiveDate::from_ymd_opt(2026, 9, 30).unwrap()),
                title: Some("Mrs".to_string()),
                first_name: "Emma".to_string(),
                middle_name: Some("Grace".to_string()),
                last_name: "Williams".to_string(),
                preferred_name: None,
                date_of_birth: NaiveDate::from_ymd_opt(1965, 3, 8).unwrap(),
                gender: Gender::Female,
                address: Address::default(),
                phone_home: Some("07 3456 7890".to_string()),
                phone_mobile: Some("0445 678 901".to_string()),
                email: Some("emma.williams@example.com".to_string()),
                emergency_contact: None,
                concession_type: None,
                concession_number: None,
                preferred_language: "English".to_string(),
                interpreter_required: false,
                aboriginal_torres_strait_islander: None,
                is_active: true,
                is_deceased: false,
                deceased_date: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            Patient {
                id: Uuid::new_v4(),
                ihi: Some("8003608166690545".to_string()),
                medicare_number: Some("6567890125".to_string()),
                medicare_irn: Some(3),
                medicare_expiry: Some(NaiveDate::from_ymd_opt(2027, 1, 31).unwrap()),
                title: Some("Mr".to_string()),
                first_name: "James".to_string(),
                middle_name: Some("Robert".to_string()),
                last_name: "Brown".to_string(),
                preferred_name: Some("Jim".to_string()),
                date_of_birth: NaiveDate::from_ymd_opt(1958, 11, 25).unwrap(),
                gender: Gender::Male,
                address: Address::default(),
                phone_home: Some("08 8234 5678".to_string()),
                phone_mobile: Some("0456 789 012".to_string()),
                email: None,
                emergency_contact: None,
                concession_type: None,
                concession_number: None,
                preferred_language: "English".to_string(),
                interpreter_required: false,
                aboriginal_torres_strait_islander: None,
                is_active: true,
                is_deceased: false,
                deceased_date: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            Patient {
                id: Uuid::new_v4(),
                ihi: Some("8003608166690552".to_string()),
                medicare_number: Some("7678901236".to_string()),
                medicare_irn: Some(1),
                medicare_expiry: Some(NaiveDate::from_ymd_opt(2025, 8, 31).unwrap()),
                title: Some("Ms".to_string()),
                first_name: "Olivia".to_string(),
                middle_name: None,
                last_name: "Martinez".to_string(),
                preferred_name: Some("Liv".to_string()),
                date_of_birth: NaiveDate::from_ymd_opt(1995, 7, 14).unwrap(),
                gender: Gender::Female,
                address: Address::default(),
                phone_home: None,
                phone_mobile: Some("0467 890 123".to_string()),
                email: Some("olivia.m@example.com".to_string()),
                emergency_contact: None,
                concession_type: None,
                concession_number: None,
                preferred_language: "English".to_string(),
                interpreter_required: false,
                aboriginal_torres_strait_islander: None,
                is_active: true,
                is_deceased: false,
                deceased_date: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            Patient {
                id: Uuid::new_v4(),
                ihi: Some("8003608166690560".to_string()),
                medicare_number: Some("8789012347".to_string()),
                medicare_irn: Some(2),
                medicare_expiry: Some(NaiveDate::from_ymd_opt(2026, 4, 30).unwrap()),
                title: Some("Mr".to_string()),
                first_name: "William".to_string(),
                middle_name: Some("Thomas".to_string()),
                last_name: "Taylor".to_string(),
                preferred_name: Some("Bill".to_string()),
                date_of_birth: NaiveDate::from_ymd_opt(1972, 9, 3).unwrap(),
                gender: Gender::Male,
                address: Address::default(),
                phone_home: Some("02 6543 2109".to_string()),
                phone_mobile: Some("0478 901 234".to_string()),
                email: Some("bill.taylor@example.com".to_string()),
                emergency_contact: None,
                concession_type: None,
                concession_number: None,
                preferred_language: "English".to_string(),
                interpreter_required: false,
                aboriginal_torres_strait_islander: None,
                is_active: true,
                is_deceased: false,
                deceased_date: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            Patient {
                id: Uuid::new_v4(),
                ihi: Some("8003608166690578".to_string()),
                medicare_number: Some("9890123458".to_string()),
                medicare_irn: Some(1),
                medicare_expiry: Some(NaiveDate::from_ymd_opt(2025, 11, 30).unwrap()),
                title: Some("Mrs".to_string()),
                first_name: "Sophia".to_string(),
                middle_name: None,
                last_name: "Anderson".to_string(),
                preferred_name: None,
                date_of_birth: NaiveDate::from_ymd_opt(1988, 2, 19).unwrap(),
                gender: Gender::Female,
                address: Address::default(),
                phone_home: None,
                phone_mobile: Some("0489 012 345".to_string()),
                email: Some("sophia.anderson@example.com".to_string()),
                emergency_contact: None,
                concession_type: None,
                concession_number: None,
                preferred_language: "English".to_string(),
                interpreter_required: false,
                aboriginal_torres_strait_islander: None,
                is_active: true,
                is_deceased: false,
                deceased_date: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            Patient {
                id: Uuid::new_v4(),
                ihi: Some("8003608166690586".to_string()),
                medicare_number: Some("1901234569".to_string()),
                medicare_irn: Some(4),
                medicare_expiry: Some(NaiveDate::from_ymd_opt(2026, 2, 28).unwrap()),
                title: Some("Mr".to_string()),
                first_name: "Liam".to_string(),
                middle_name: Some("Patrick".to_string()),
                last_name: "O'Connor".to_string(),
                preferred_name: None,
                date_of_birth: NaiveDate::from_ymd_opt(2010, 4, 12).unwrap(),
                gender: Gender::Male,
                address: Address::default(),
                phone_home: Some("03 9123 4567".to_string()),
                phone_mobile: None,
                email: None,
                emergency_contact: None,
                concession_type: None,
                concession_number: None,
                preferred_language: "English".to_string(),
                interpreter_required: false,
                aboriginal_torres_strait_islander: None,
                is_active: true,
                is_deceased: false,
                deceased_date: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            Patient {
                id: Uuid::new_v4(),
                ihi: Some("8003608166690594".to_string()),
                medicare_number: Some("2012345670".to_string()),
                medicare_irn: Some(1),
                medicare_expiry: Some(NaiveDate::from_ymd_opt(2027, 5, 31).unwrap()),
                title: Some("Ms".to_string()),
                first_name: "Ava".to_string(),
                middle_name: None,
                last_name: "Nguyen".to_string(),
                preferred_name: None,
                date_of_birth: NaiveDate::from_ymd_opt(1998, 6, 21).unwrap(),
                gender: Gender::Female,
                address: Address::default(),
                phone_home: None,
                phone_mobile: Some("0401 234 567".to_string()),
                email: Some("ava.nguyen@example.com".to_string()),
                emergency_contact: None,
                concession_type: None,
                concession_number: None,
                preferred_language: "English".to_string(),
                interpreter_required: false,
                aboriginal_torres_strait_islander: None,
                is_active: true,
                is_deceased: false,
                deceased_date: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            Patient {
                id: Uuid::new_v4(),
                ihi: Some("8003608166690602".to_string()),
                medicare_number: Some("3123456781".to_string()),
                medicare_irn: Some(2),
                medicare_expiry: Some(NaiveDate::from_ymd_opt(2025, 7, 31).unwrap()),
                title: Some("Mr".to_string()),
                first_name: "Noah".to_string(),
                middle_name: Some("Alexander".to_string()),
                last_name: "Davis".to_string(),
                preferred_name: None,
                date_of_birth: NaiveDate::from_ymd_opt(1963, 1, 30).unwrap(),
                gender: Gender::Male,
                address: Address::default(),
                phone_home: Some("07 5432 1098".to_string()),
                phone_mobile: Some("0412 345 678".to_string()),
                email: Some("noah.davis@example.com".to_string()),
                emergency_contact: None,
                concession_type: None,
                concession_number: None,
                preferred_language: "English".to_string(),
                interpreter_required: false,
                aboriginal_torres_strait_islander: None,
                is_active: true,
                is_deceased: false,
                deceased_date: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            Patient {
                id: Uuid::new_v4(),
                ihi: Some("8003608166690610".to_string()),
                medicare_number: Some("4234567892".to_string()),
                medicare_irn: Some(1),
                medicare_expiry: Some(NaiveDate::from_ymd_opt(2026, 10, 31).unwrap()),
                title: Some("Mrs".to_string()),
                first_name: "Isabella".to_string(),
                middle_name: Some("Rose".to_string()),
                last_name: "Wilson".to_string(),
                preferred_name: Some("Bella".to_string()),
                date_of_birth: NaiveDate::from_ymd_opt(1970, 10, 5).unwrap(),
                gender: Gender::Female,
                address: Address::default(),
                phone_home: Some("08 9876 5432".to_string()),
                phone_mobile: Some("0423 456 789".to_string()),
                email: Some("isabella.wilson@example.com".to_string()),
                emergency_contact: None,
                concession_type: None,
                concession_number: None,
                preferred_language: "English".to_string(),
                interpreter_required: false,
                aboriginal_torres_strait_islander: None,
                is_active: true,
                is_deceased: false,
                deceased_date: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            Patient {
                id: Uuid::new_v4(),
                ihi: Some("8003608166690628".to_string()),
                medicare_number: Some("5345678903".to_string()),
                medicare_irn: Some(3),
                medicare_expiry: Some(NaiveDate::from_ymd_opt(2027, 3, 31).unwrap()),
                title: Some("Mr".to_string()),
                first_name: "Mason".to_string(),
                middle_name: None,
                last_name: "Moore".to_string(),
                preferred_name: None,
                date_of_birth: NaiveDate::from_ymd_opt(1985, 12, 18).unwrap(),
                gender: Gender::Male,
                address: Address::default(),
                phone_home: None,
                phone_mobile: Some("0434 567 890".to_string()),
                email: Some("mason.moore@example.com".to_string()),
                emergency_contact: None,
                concession_type: None,
                concession_number: None,
                preferred_language: "English".to_string(),
                interpreter_required: false,
                aboriginal_torres_strait_islander: None,
                is_active: true,
                is_deceased: false,
                deceased_date: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            Patient {
                id: Uuid::new_v4(),
                ihi: Some("8003608166690636".to_string()),
                medicare_number: Some("6456789014".to_string()),
                medicare_irn: Some(1),
                medicare_expiry: Some(NaiveDate::from_ymd_opt(2025, 10, 31).unwrap()),
                title: Some("Ms".to_string()),
                first_name: "Mia".to_string(),
                middle_name: Some("Charlotte".to_string()),
                last_name: "Lee".to_string(),
                preferred_name: None,
                date_of_birth: NaiveDate::from_ymd_opt(2005, 5, 27).unwrap(),
                gender: Gender::Female,
                address: Address::default(),
                phone_home: Some("02 4321 0987".to_string()),
                phone_mobile: Some("0445 678 901".to_string()),
                email: Some("mia.lee@example.com".to_string()),
                emergency_contact: None,
                concession_type: None,
                concession_number: None,
                preferred_language: "English".to_string(),
                interpreter_required: false,
                aboriginal_torres_strait_islander: None,
                is_active: true,
                is_deceased: false,
                deceased_date: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            Patient {
                id: Uuid::new_v4(),
                ihi: Some("8003608166690644".to_string()),
                medicare_number: Some("7567890125".to_string()),
                medicare_irn: Some(2),
                medicare_expiry: Some(NaiveDate::from_ymd_opt(2026, 8, 31).unwrap()),
                title: Some("Mr".to_string()),
                first_name: "Ethan".to_string(),
                middle_name: Some("Michael".to_string()),
                last_name: "White".to_string(),
                preferred_name: None,
                date_of_birth: NaiveDate::from_ymd_opt(1991, 8, 9).unwrap(),
                gender: Gender::Male,
                address: Address::default(),
                phone_home: None,
                phone_mobile: Some("0456 789 012".to_string()),
                email: Some("ethan.white@example.com".to_string()),
                emergency_contact: None,
                concession_type: None,
                concession_number: None,
                preferred_language: "English".to_string(),
                interpreter_required: false,
                aboriginal_torres_strait_islander: None,
                is_active: true,
                is_deceased: false,
                deceased_date: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            Patient {
                id: Uuid::new_v4(),
                ihi: Some("8003608166690651".to_string()),
                medicare_number: Some("8678901236".to_string()),
                medicare_irn: Some(1),
                medicare_expiry: Some(NaiveDate::from_ymd_opt(2027, 2, 28).unwrap()),
                title: Some("Mrs".to_string()),
                first_name: "Charlotte".to_string(),
                middle_name: None,
                last_name: "Harris".to_string(),
                preferred_name: Some("Charlie".to_string()),
                date_of_birth: NaiveDate::from_ymd_opt(1955, 4, 16).unwrap(),
                gender: Gender::Female,
                address: Address::default(),
                phone_home: Some("03 5678 9012".to_string()),
                phone_mobile: Some("0467 890 123".to_string()),
                email: None,
                emergency_contact: None,
                concession_type: None,
                concession_number: None,
                preferred_language: "English".to_string(),
                interpreter_required: false,
                aboriginal_torres_strait_islander: None,
                is_active: true,
                is_deceased: false,
                deceased_date: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            Patient {
                id: Uuid::new_v4(),
                ihi: Some("8003608166690669".to_string()),
                medicare_number: Some("9789012347".to_string()),
                medicare_irn: Some(4),
                medicare_expiry: Some(NaiveDate::from_ymd_opt(2025, 9, 30).unwrap()),
                title: Some("Mr".to_string()),
                first_name: "Benjamin".to_string(),
                middle_name: Some("Oliver".to_string()),
                last_name: "Martin".to_string(),
                preferred_name: Some("Ben".to_string()),
                date_of_birth: NaiveDate::from_ymd_opt(2015, 3, 7).unwrap(),
                gender: Gender::Male,
                address: Address::default(),
                phone_home: Some("07 8901 2345".to_string()),
                phone_mobile: None,
                email: None,
                emergency_contact: None,
                concession_type: None,
                concession_number: None,
                preferred_language: "English".to_string(),
                interpreter_required: false,
                aboriginal_torres_strait_islander: None,
                is_active: true,
                is_deceased: false,
                deceased_date: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            Patient {
                id: Uuid::new_v4(),
                ihi: Some("8003608166690677".to_string()),
                medicare_number: Some("1890123458".to_string()),
                medicare_irn: Some(1),
                medicare_expiry: Some(NaiveDate::from_ymd_opt(2026, 11, 30).unwrap()),
                title: Some("Ms".to_string()),
                first_name: "Amelia".to_string(),
                middle_name: None,
                last_name: "Thompson".to_string(),
                preferred_name: Some("Amy".to_string()),
                date_of_birth: NaiveDate::from_ymd_opt(1993, 11, 23).unwrap(),
                gender: Gender::Female,
                address: Address::default(),
                phone_home: None,
                phone_mobile: Some("0478 901 234".to_string()),
                email: Some("amelia.thompson@example.com".to_string()),
                emergency_contact: None,
                concession_type: None,
                concession_number: None,
                preferred_language: "English".to_string(),
                interpreter_required: false,
                aboriginal_torres_strait_islander: None,
                is_active: true,
                is_deceased: false,
                deceased_date: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            Patient {
                id: Uuid::new_v4(),
                ihi: Some("8003608166690685".to_string()),
                medicare_number: Some("2901234569".to_string()),
                medicare_irn: Some(2),
                medicare_expiry: Some(NaiveDate::from_ymd_opt(2027, 6, 30).unwrap()),
                title: Some("Mr".to_string()),
                first_name: "Lucas".to_string(),
                middle_name: Some("William".to_string()),
                last_name: "Garcia".to_string(),
                preferred_name: None,
                date_of_birth: NaiveDate::from_ymd_opt(1982, 7, 11).unwrap(),
                gender: Gender::Male,
                address: Address::default(),
                phone_home: Some("08 7654 3210".to_string()),
                phone_mobile: Some("0489 012 345".to_string()),
                email: Some("lucas.garcia@example.com".to_string()),
                emergency_contact: None,
                concession_type: None,
                concession_number: None,
                preferred_language: "English".to_string(),
                interpreter_required: false,
                aboriginal_torres_strait_islander: None,
                is_active: true,
                is_deceased: false,
                deceased_date: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            Patient {
                id: Uuid::new_v4(),
                ihi: Some("8003608166690693".to_string()),
                medicare_number: Some("3012345670".to_string()),
                medicare_irn: Some(1),
                medicare_expiry: Some(NaiveDate::from_ymd_opt(2025, 5, 31).unwrap()),
                title: Some("Mrs".to_string()),
                first_name: "Harper".to_string(),
                middle_name: Some("Elizabeth".to_string()),
                last_name: "Robinson".to_string(),
                preferred_name: None,
                date_of_birth: NaiveDate::from_ymd_opt(1968, 9, 14).unwrap(),
                gender: Gender::Female,
                address: Address::default(),
                phone_home: Some("02 3210 9876".to_string()),
                phone_mobile: Some("0401 234 567".to_string()),
                email: Some("harper.robinson@example.com".to_string()),
                emergency_contact: None,
                concession_type: None,
                concession_number: None,
                preferred_language: "English".to_string(),
                interpreter_required: false,
                aboriginal_torres_strait_islander: None,
                is_active: true,
                is_deceased: false,
                deceased_date: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
        ]
    }

    fn select_next(&mut self) {
        if self.filtered_patients.is_empty() {
            return;
        }
        
        let current = self.table_state.selected().unwrap_or(0);
        let next = (current + 1).min(self.filtered_patients.len() - 1);
        self.table_state.select(Some(next));
    }

    fn select_previous(&mut self) {
        if self.filtered_patients.is_empty() {
            return;
        }
        
        let current = self.table_state.selected().unwrap_or(0);
        let prev = current.saturating_sub(1);
        self.table_state.select(Some(prev));
    }

    fn select_first(&mut self) {
        if !self.filtered_patients.is_empty() {
            self.table_state.select(Some(0));
        }
    }

    fn select_last(&mut self) {
        if !self.filtered_patients.is_empty() {
            self.table_state.select(Some(self.filtered_patients.len() - 1));
        }
    }

    fn selected_patient(&self) -> Option<&Patient> {
        self.table_state
            .selected()
            .and_then(|i| self.filtered_patients.get(i))
    }

    fn apply_search_filter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_patients = self.all_patients.clone();
        } else {
            let query = self.search_query.to_lowercase();
            self.filtered_patients = self
                .all_patients
                .iter()
                .filter(|p| {
                    let full_name = format!("{} {}", p.first_name, p.last_name).to_lowercase();
                    let preferred = p
                        .preferred_name
                        .as_ref()
                        .map(|n| n.to_lowercase())
                        .unwrap_or_default();
                    let medicare = p
                        .medicare_number
                        .as_ref()
                        .map(|m| m.to_lowercase())
                        .unwrap_or_default();

                    full_name.contains(&query)
                        || preferred.contains(&query)
                        || medicare.contains(&query)
                })
                .cloned()
                .collect();
        }
        
        if !self.filtered_patients.is_empty() {
            self.table_state.select(Some(0));
        } else {
            self.table_state.select(None);
        }
    }

    fn enter_search_mode(&mut self) {
        self.search_mode = true;
        self.search_query.clear();
    }

    fn exit_search_mode(&mut self) {
        self.search_mode = false;
    }

    fn handle_search_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.apply_search_filter();
                Action::Render
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.apply_search_filter();
                Action::Render
            }
            KeyCode::Enter | KeyCode::Esc => {
                self.exit_search_mode();
                Action::Render
            }
            _ => Action::None,
        }
    }

    fn render_search_bar_static(
        frame: &mut Frame,
        area: Rect,
        search_query: &str,
        search_mode: bool,
    ) {
        use ratatui::widgets::Paragraph;

        let search_text = if search_mode {
            format!("Search: {}█", search_query)
        } else {
            format!("Filter: {} (/ to edit, Esc to clear)", search_query)
        };

        let search_style = if search_mode {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Green)
        };

        let search_bar = Paragraph::new(search_text)
            .style(search_style)
            .block(Block::default().borders(Borders::ALL).title(" Search "));

        frame.render_widget(search_bar, area);
    }
}

#[async_trait]
impl Component for PatientListComponent {
    async fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        if self.search_mode {
            return self.handle_search_input(key);
        }

        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.select_next();
                Action::Render
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.select_previous();
                Action::Render
            }
            KeyCode::Char('g') => {
                self.select_first();
                Action::Render
            }
            KeyCode::Char('G') => {
                self.select_last();
                Action::Render
            }
            KeyCode::Enter => {
                if let Some(_patient) = self.selected_patient() {
                    Action::None
                } else {
                    Action::None
                }
            }
            KeyCode::Char('n') => {
                Action::None
            }
            KeyCode::Char('/') => {
                self.enter_search_mode();
                Action::Render
            }
            KeyCode::Esc => {
                if !self.search_query.is_empty() {
                    self.search_query.clear();
                    self.apply_search_filter();
                    Action::Render
                } else {
                    Action::None
                }
            }
            _ => Action::None,
        }
    }

    async fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => {
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        use ratatui::layout::{Constraint as LayoutConstraint, Direction, Layout};

        let table_area = if self.search_mode || !self.search_query.is_empty() {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    LayoutConstraint::Length(3),
                    LayoutConstraint::Min(0),
                ])
                .split(area);
            Self::render_search_bar_static(frame, chunks[0], &self.search_query, self.search_mode);
            chunks[1]
        } else {
            area
        };

        let header_style = Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD);

        let header = Row::new(vec![
            Cell::from("Name"),
            Cell::from("DOB"),
            Cell::from("Age"),
            Cell::from("Medicare"),
            Cell::from("Phone"),
        ])
        .style(header_style)
        .height(1);

        let rows: Vec<Row> = self
            .filtered_patients
            .iter()
            .map(|patient| {
                let name = format!(
                    "{}, {}",
                    patient.last_name,
                    patient.preferred_name
                        .as_ref()
                        .unwrap_or(&patient.first_name)
                );
                let dob = patient.date_of_birth.format("%d/%m/%Y").to_string();
                let age = patient.age().to_string();
                let medicare = patient
                    .medicare_number
                    .as_ref()
                    .map(|m| {
                        if let Some(irn) = patient.medicare_irn {
                            format!("{}-{}", m, irn)
                        } else {
                            m.clone()
                        }
                    })
                    .unwrap_or_else(|| "-".to_string());
                let phone = patient
                    .phone_mobile
                    .as_ref()
                    .or(patient.phone_home.as_ref())
                    .map(|p| p.clone())
                    .unwrap_or_else(|| "-".to_string());

                Row::new(vec![
                    Cell::from(name),
                    Cell::from(dob),
                    Cell::from(age),
                    Cell::from(medicare),
                    Cell::from(phone),
                ])
                .height(1)
            })
            .collect();

        let widths = [
            Constraint::Percentage(30),
            Constraint::Percentage(15),
            Constraint::Percentage(10),
            Constraint::Percentage(25),
            Constraint::Percentage(20),
        ];

        let title = if self.search_query.is_empty() {
            " Patient List (j/k:navigate g/G:first/last /:search Esc:clear) ".to_string()
        } else {
            format!(
                " Patient List - {} results (Esc:clear) ",
                self.filtered_patients.len()
            )
        };

        let table = Table::new(rows, widths)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title(title))
            .row_highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(table, table_area, &mut self.table_state);

        if let Some(ref error) = self.error_message {
            let error_text = format!("Error: {}", error);
            let error_paragraph = ratatui::widgets::Paragraph::new(error_text)
                .style(Style::default().fg(Color::Red));
            frame.render_widget(error_paragraph, table_area);
        }
    }
}
