use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathologyOrder {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub ordering_practitioner_id: Uuid,
    pub consultation_id: Option<Uuid>,

    pub order_number: String,
    pub order_date: DateTime<Utc>,
    pub collection_date: Option<NaiveDate>,

    pub laboratory: Laboratory,
    pub tests: Vec<TestRequest>,

    pub clinical_notes: Option<String>,
    pub urgent: bool,
    pub fasting_required: bool,

    pub status: OrderStatus,
    pub hl7_message_sent: bool,
    pub hl7_message_id: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRequest {
    pub test_name: String,
    pub test_code: Option<String>,
    pub loinc_code: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum OrderStatus {
    Draft,
    Ordered,
    Collected,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathologyResult {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub order_id: Option<Uuid>,

    pub laboratory: Laboratory,
    pub lab_report_number: String,
    pub collection_date: NaiveDate,
    pub report_date: DateTime<Utc>,

    pub tests: Vec<TestResult>,
    pub clinical_notes: Option<String>,
    pub pathologist_comment: Option<String>,

    pub has_abnormal: bool,
    pub has_critical: bool,

    pub acknowledged: bool,
    pub acknowledged_by: Option<Uuid>,
    pub acknowledged_at: Option<DateTime<Utc>>,

    pub hl7_message_received: bool,
    pub hl7_message_id: Option<String>,
    pub pdf_report_path: Option<String>,

    pub received_at: DateTime<Utc>,
}

impl PathologyResult {
    pub fn check_abnormal_flags(&mut self) {
        self.has_abnormal = self.tests.iter().any(|t| t.flag.is_some());
        self.has_critical = self.tests.iter().any(|t| {
            matches!(
                t.flag,
                Some(ResultFlag::Critical)
                    | Some(ResultFlag::CriticalHigh)
                    | Some(ResultFlag::CriticalLow)
            )
        });
    }

    pub fn acknowledge(&mut self, user_id: Uuid) {
        self.acknowledged = true;
        self.acknowledged_by = Some(user_id);
        self.acknowledged_at = Some(Utc::now());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_name: String,
    pub test_code: Option<String>,
    pub loinc_code: Option<String>,
    pub value: String,
    pub unit: Option<String>,
    pub reference_range: Option<String>,
    pub flag: Option<ResultFlag>,
    pub status: ResultStatus,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum ResultFlag {
    Normal,
    High,
    Low,
    CriticalHigh,
    CriticalLow,
    Critical,
    Abnormal,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum ResultStatus {
    Final,
    Preliminary,
    Corrected,
    Cancelled,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum Laboratory {
    ACL,
    Sonic,
    Healius,
    QML,
    DouglassHanlyMoir,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImagingOrder {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub ordering_practitioner_id: Uuid,
    pub consultation_id: Option<Uuid>,

    pub order_number: String,
    pub order_date: DateTime<Utc>,
    pub appointment_date: Option<NaiveDate>,

    pub imaging_provider: String,
    pub modality: ImagingModality,
    pub body_part: String,
    pub clinical_indication: String,

    pub urgent: bool,
    pub contrast_required: bool,

    pub status: OrderStatus,

    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum ImagingModality {
    XRay,
    CT,
    MRI,
    Ultrasound,
    NuclearMedicine,
    PET,
    Mammography,
    DXA,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImagingResult {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub order_id: Option<Uuid>,

    pub imaging_provider: String,
    pub report_number: String,
    pub study_date: NaiveDate,
    pub report_date: DateTime<Utc>,

    pub modality: ImagingModality,
    pub body_part: String,
    pub findings: String,
    pub impression: String,
    pub radiologist_name: Option<String>,

    pub has_significant_findings: bool,

    pub acknowledged: bool,
    pub acknowledged_by: Option<Uuid>,
    pub acknowledged_at: Option<DateTime<Utc>>,

    pub pdf_report_path: Option<String>,
    pub dicom_available: bool,

    pub received_at: DateTime<Utc>,
}
