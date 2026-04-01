#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MbsItem {
    pub item_num: i32,
    pub sub_item_num: Option<i32>,
    pub item_start_date: Option<String>,
    pub item_end_date: Option<String>,
    pub category: Option<String>,
    pub group_code: Option<String>,
    pub sub_group: Option<String>,
    pub sub_heading: Option<String>,
    pub item_type: Option<String>,
    pub fee_type: Option<String>,
    pub provider_type: Option<String>,
    pub schedule_fee: Option<f64>,
    pub benefit_75: Option<f64>,
    pub benefit_85: Option<f64>,
    pub benefit_100: Option<f64>,
    pub derived_fee: Option<String>,
    pub description: Option<String>,
    pub description_start_date: Option<String>,
    pub emsn_cap: Option<String>,
    pub emsn_maximum_cap: Option<f64>,
    pub emsn_percentage_cap: Option<f64>,
    pub is_gst_free: bool,
    pub is_active: bool,
    pub imported_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MbsImportResult {
    pub total_imported: u32,
    pub updated: u32,
    pub skipped: u32,
    pub errors: Vec<String>,
}
