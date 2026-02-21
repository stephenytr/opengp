use crate::domain::clinical::Consultation;
use ratatui::prelude::*;

pub struct ConsultationDetail {
    pub consultation: Option<Consultation>,
    pub is_editing: bool,
    pub signed: bool,
}

#[derive(Debug, Clone)]
pub enum ConsultationDetailAction {
    Edit,
    Save,
    Sign,
    Cancel,
}

impl ConsultationDetail {
    pub fn new() -> Self {
        Self {
            consultation: None,
            is_editing: false,
            signed: false,
        }
    }

    pub fn with_consultation(consultation: Consultation) -> Self {
        let signed = consultation.signed_at.is_some();
        Self {
            consultation: Some(consultation),
            is_editing: false,
            signed,
        }
    }

    pub fn can_edit(&self) -> bool {
        !self.signed
    }

    pub fn start_editing(&mut self) {
        if self.can_edit() {
            self.is_editing = true;
        }
    }

    pub fn stop_editing(&mut self) {
        self.is_editing = false;
    }
}
