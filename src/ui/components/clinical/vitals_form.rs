use crate::domain::clinical::VitalSigns;
use ratatui::prelude::*;

#[derive(Debug, Clone)]
pub enum VitalSignsFormField {
    SystolicBp,
    DiastolicBp,
    HeartRate,
    RespiratoryRate,
    Temperature,
    O2Saturation,
    Height,
    Weight,
    Notes,
}

pub struct VitalSignsForm {
    pub systolic_bp: Option<u16>,
    pub diastolic_bp: Option<u16>,
    pub heart_rate: Option<u16>,
    pub respiratory_rate: Option<u16>,
    pub temperature: Option<f32>,
    pub o2_saturation: Option<u8>,
    pub height_cm: Option<u16>,
    pub weight_kg: Option<f32>,
    pub notes: String,
    pub focused_field: VitalSignsFormField,
    pub calculated_bmi: Option<f32>,
}

#[derive(Debug, Clone)]
pub enum VitalSignsFormAction {
    Save,
    Cancel,
    NextField,
    PrevField,
}

impl VitalSignsForm {
    pub fn new() -> Self {
        Self {
            systolic_bp: None,
            diastolic_bp: None,
            heart_rate: None,
            respiratory_rate: None,
            temperature: None,
            o2_saturation: None,
            height_cm: None,
            weight_kg: None,
            notes: String::new(),
            focused_field: VitalSignsFormField::SystolicBp,
            calculated_bmi: None,
        }
    }

    pub fn calculate_bmi(&mut self) {
        if let (Some(height), Some(weight)) = (self.height_cm, self.weight_kg) {
            if height > 0 {
                let height_m = height as f32 / 100.0;
                self.calculated_bmi = Some(weight / (height_m * height_m));
            }
        } else {
            self.calculated_bmi = None;
        }
    }

    pub fn to_vital_signs(&self, patient_id: uuid::Uuid, created_by: uuid::Uuid) -> VitalSigns {
        VitalSigns {
            id: uuid::Uuid::new_v4(),
            patient_id,
            consultation_id: None,
            measured_at: chrono::Utc::now(),
            systolic_bp: self.systolic_bp,
            diastolic_bp: self.diastolic_bp,
            heart_rate: self.heart_rate,
            respiratory_rate: self.respiratory_rate,
            temperature: self.temperature,
            oxygen_saturation: self.o2_saturation,
            height_cm: self.height_cm,
            weight_kg: self.weight_kg,
            bmi: self.calculated_bmi,
            notes: Some(self.notes.clone()).filter(|s| !s.is_empty()),
            created_at: chrono::Utc::now(),
            created_by,
        }
    }
}
