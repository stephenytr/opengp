use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use opengp_domain::domain::billing::{MbsImportResult, MbsItem};
use sqlx::{FromRow, PgPool};
use thiserror::Error;

use super::xml_parser::{parse_mbs_xml_reader, MbsXmlParseError};

#[derive(Debug, Error)]
pub enum MbsImportError {
    #[error("I/O error while importing MBS XML: {0}")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Parse(#[from] MbsXmlParseError),
    #[error("Database error while importing MBS data: {0}")]
    Database(#[from] sqlx::Error),
}

#[derive(Debug, FromRow)]
struct MbsItemRow {
    item_num: i32,
    sub_item_num: Option<i32>,
    item_start_date: Option<String>,
    item_end_date: Option<String>,
    category: Option<String>,
    group_code: Option<String>,
    sub_group: Option<String>,
    sub_heading: Option<String>,
    item_type: Option<String>,
    fee_type: Option<String>,
    provider_type: Option<String>,
    schedule_fee: Option<f64>,
    benefit_75: Option<f64>,
    benefit_85: Option<f64>,
    benefit_100: Option<f64>,
    derived_fee: Option<String>,
    description: Option<String>,
    description_start_date: Option<String>,
    emsn_cap: Option<String>,
    emsn_maximum_cap: Option<f64>,
    emsn_percentage_cap: Option<f64>,
    is_gst_free: bool,
    is_active: bool,
    imported_at: String,
}

impl MbsItemRow {
    fn into_domain(self) -> MbsItem {
        MbsItem {
            item_num: self.item_num,
            sub_item_num: self.sub_item_num,
            item_start_date: self.item_start_date,
            item_end_date: self.item_end_date,
            category: self.category,
            group_code: self.group_code,
            sub_group: self.sub_group,
            sub_heading: self.sub_heading,
            item_type: self.item_type,
            fee_type: self.fee_type,
            provider_type: self.provider_type,
            schedule_fee: self.schedule_fee,
            benefit_75: self.benefit_75,
            benefit_85: self.benefit_85,
            benefit_100: self.benefit_100,
            derived_fee: self.derived_fee,
            description: self.description,
            description_start_date: self.description_start_date,
            emsn_cap: self.emsn_cap,
            emsn_maximum_cap: self.emsn_maximum_cap,
            emsn_percentage_cap: self.emsn_percentage_cap,
            is_gst_free: self.is_gst_free,
            is_active: self.is_active,
            imported_at: self.imported_at,
        }
    }
}

pub struct SqlxMbsRepository {
    pool: PgPool,
}

impl SqlxMbsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn import_items<P: AsRef<Path>>(
        &self,
        xml_path: P,
    ) -> Result<MbsImportResult, MbsImportError> {
        let file = File::open(xml_path)?;
        let reader = BufReader::new(file);
        let items = parse_mbs_xml_reader(reader)?;

        let mut tx = self.pool.begin().await?;
        let mut updated: u32 = 0;

        for item in &items {
            let existing =
                sqlx::query_scalar::<_, i64>("SELECT COUNT(1) FROM mbs_items WHERE item_num = $1")
                    .bind(item.item_num)
                    .fetch_one(&mut *tx)
                    .await?;

            if existing > 0 {
                updated += 1;
            }

            sqlx::query(
                r#"
                INSERT INTO mbs_items (
                    item_num, sub_item_num, item_start_date, item_end_date,
                    category, group_code, sub_group, sub_heading,
                    item_type, fee_type, provider_type,
                    schedule_fee, benefit_75, benefit_85, benefit_100,
                    derived_fee, description, description_start_date,
                    emsn_cap, emsn_maximum_cap, emsn_percentage_cap,
                    is_gst_free, is_active, imported_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24)
                ON CONFLICT (item_num) DO UPDATE SET
                  sub_item_num = EXCLUDED.sub_item_num,
                  item_start_date = EXCLUDED.item_start_date,
                  item_end_date = EXCLUDED.item_end_date,
                  category = EXCLUDED.category,
                  group_code = EXCLUDED.group_code,
                  sub_group = EXCLUDED.sub_group,
                  sub_heading = EXCLUDED.sub_heading,
                  item_type = EXCLUDED.item_type,
                  fee_type = EXCLUDED.fee_type,
                  provider_type = EXCLUDED.provider_type,
                  schedule_fee = EXCLUDED.schedule_fee,
                  benefit_75 = EXCLUDED.benefit_75,
                  benefit_85 = EXCLUDED.benefit_85,
                  benefit_100 = EXCLUDED.benefit_100,
                  derived_fee = EXCLUDED.derived_fee,
                  description = EXCLUDED.description,
                  description_start_date = EXCLUDED.description_start_date,
                  emsn_cap = EXCLUDED.emsn_cap,
                  emsn_maximum_cap = EXCLUDED.emsn_maximum_cap,
                  emsn_percentage_cap = EXCLUDED.emsn_percentage_cap,
                  is_gst_free = EXCLUDED.is_gst_free,
                  is_active = EXCLUDED.is_active,
                  imported_at = EXCLUDED.imported_at
                "#,
            )
            .bind(item.item_num)
            .bind(item.sub_item_num)
            .bind(item.item_start_date.as_deref())
            .bind(item.item_end_date.as_deref())
            .bind(item.category.as_deref())
            .bind(item.group_code.as_deref())
            .bind(item.sub_group.as_deref())
            .bind(item.sub_heading.as_deref())
            .bind(item.item_type.as_deref())
            .bind(item.fee_type.as_deref())
            .bind(item.provider_type.as_deref())
            .bind(item.schedule_fee)
            .bind(item.benefit_75)
            .bind(item.benefit_85)
            .bind(item.benefit_100)
            .bind(item.derived_fee.as_deref())
            .bind(item.description.as_deref())
            .bind(item.description_start_date.as_deref())
            .bind(item.emsn_cap.as_deref())
            .bind(item.emsn_maximum_cap)
            .bind(item.emsn_percentage_cap)
            .bind(item.is_gst_free)
            .bind(item.is_active)
            .bind(item.imported_at.as_str())
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(MbsImportResult {
            total_imported: items.len() as u32,
            updated,
            skipped: 0,
            errors: Vec::new(),
        })
    }

    pub async fn find_by_item_num(&self, item_num: i32) -> Result<Option<MbsItem>, MbsImportError> {
        let row = sqlx::query_as::<_, MbsItemRow>(
            r#"
            SELECT
                item_num, sub_item_num, item_start_date, item_end_date,
                category, group_code, sub_group, sub_heading,
                item_type, fee_type, provider_type,
                schedule_fee, benefit_75, benefit_85, benefit_100,
                derived_fee, description, description_start_date,
                emsn_cap, emsn_maximum_cap, emsn_percentage_cap,
                is_gst_free, is_active, imported_at
            FROM mbs_items
            WHERE item_num = $1
            "#,
        )
        .bind(item_num)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(MbsItemRow::into_domain))
    }

    pub async fn search(&self, query: &str, limit: i64) -> Result<Vec<MbsItem>, MbsImportError> {
        let pattern = format!("%{}%", query);

        let rows = sqlx::query_as::<_, MbsItemRow>(
            r#"
            SELECT
                item_num, sub_item_num, item_start_date, item_end_date,
                category, group_code, sub_group, sub_heading,
                item_type, fee_type, provider_type,
                schedule_fee, benefit_75, benefit_85, benefit_100,
                derived_fee, description, description_start_date,
                emsn_cap, emsn_maximum_cap, emsn_percentage_cap,
                is_gst_free, is_active, imported_at
            FROM mbs_items
            WHERE is_active = true AND (item_num::text LIKE $1 OR description LIKE $2)
            ORDER BY item_num
            LIMIT $3
            "#,
        )
        .bind(&pattern)
        .bind(&pattern)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(MbsItemRow::into_domain).collect())
    }

    pub async fn list_active(&self) -> Result<Vec<MbsItem>, MbsImportError> {
        let rows = sqlx::query_as::<_, MbsItemRow>(
            r#"
            SELECT
                item_num, sub_item_num, item_start_date, item_end_date,
                category, group_code, sub_group, sub_heading,
                item_type, fee_type, provider_type,
                schedule_fee, benefit_75, benefit_85, benefit_100,
                derived_fee, description, description_start_date,
                emsn_cap, emsn_maximum_cap, emsn_percentage_cap,
                is_gst_free, is_active, imported_at
            FROM mbs_items
            WHERE is_active = true
            ORDER BY item_num
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(MbsItemRow::into_domain).collect())
    }

    pub async fn count(&self) -> Result<i64, MbsImportError> {
        let total =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(1) FROM mbs_items WHERE is_active = true")
                .fetch_one(&self.pool)
                .await?;

        Ok(total)
    }
}
