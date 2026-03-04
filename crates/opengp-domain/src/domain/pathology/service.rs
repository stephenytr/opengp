use std::sync::Arc;

use crate::service;

use super::error::{ServiceError, ValidationError};
use super::model::{OrderStatus, PathologyOrder, PathologyResult};
use super::repository::PathologyRepository;

service! {
    PathologyService {
        repository: Arc<dyn PathologyRepository>,
    }
}

impl PathologyService {
    fn validate_order(&self, order: &PathologyOrder) -> Result<(), ServiceError> {
        if order.order_number.trim().is_empty() {
            return Err(ValidationError::EmptyOrderNumber.into());
        }

        if order.tests.is_empty() {
            return Err(ValidationError::EmptyTestList.into());
        }

        Ok(())
    }

    pub async fn create_order(&self, order: PathologyOrder) -> Result<PathologyOrder, ServiceError> {
        self.validate_order(&order)?;
        Ok(self.repository.create_order(order).await?)
    }

    pub async fn create_result(
        &self,
        mut result: PathologyResult,
    ) -> Result<PathologyResult, ServiceError> {
        result.check_abnormal_flags();
        Ok(self.repository.create_result(result).await?)
    }

    pub async fn find_orders_by_status(
        &self,
        status: OrderStatus,
    ) -> Result<Vec<PathologyOrder>, ServiceError> {
        Ok(self.repository.find_orders_by_status(status).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::pathology::{
        Laboratory, RepositoryError, ResultFlag, ResultStatus, TestRequest, TestResult,
    };
    use async_trait::async_trait;
    use chrono::{NaiveDate, Utc};
    use uuid::Uuid;

    struct MockPathologyRepository {
        orders: Vec<PathologyOrder>,
    }

    #[async_trait]
    impl PathologyRepository for MockPathologyRepository {
        async fn find_order_by_id(&self, id: Uuid) -> Result<Option<PathologyOrder>, RepositoryError> {
            Ok(self.orders.iter().find(|order| order.id == id).cloned())
        }

        async fn find_result_by_id(&self, _id: Uuid) -> Result<Option<PathologyResult>, RepositoryError> {
            Ok(None)
        }

        async fn find_orders_by_patient(
            &self,
            patient_id: Uuid,
        ) -> Result<Vec<PathologyOrder>, RepositoryError> {
            Ok(self
                .orders
                .iter()
                .filter(|order| order.patient_id == patient_id)
                .cloned()
                .collect())
        }

        async fn create_order(&self, order: PathologyOrder) -> Result<PathologyOrder, RepositoryError> {
            Ok(order)
        }

        async fn update_order(&self, order: PathologyOrder) -> Result<PathologyOrder, RepositoryError> {
            Ok(order)
        }

        async fn create_result(&self, result: PathologyResult) -> Result<PathologyResult, RepositoryError> {
            Ok(result)
        }

        async fn find_orders_by_status(
            &self,
            status: OrderStatus,
        ) -> Result<Vec<PathologyOrder>, RepositoryError> {
            Ok(self
                .orders
                .iter()
                .filter(|order| order.status == status)
                .cloned()
                .collect())
        }
    }

    fn new_service(orders: Vec<PathologyOrder>) -> PathologyService {
        PathologyService::new(Arc::new(MockPathologyRepository { orders }))
    }

    fn test_order(status: OrderStatus) -> PathologyOrder {
        PathologyOrder {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            ordering_practitioner_id: Uuid::new_v4(),
            consultation_id: None,
            order_number: "PATH-0001".to_string(),
            order_date: Utc::now(),
            collection_date: Some(NaiveDate::from_ymd_opt(2026, 3, 1).expect("valid date")),
            laboratory: Laboratory::Sonic,
            tests: vec![TestRequest {
                test_name: "Full Blood Count".to_string(),
                test_code: Some("FBC".to_string()),
                loinc_code: None,
            }],
            clinical_notes: None,
            urgent: false,
            fasting_required: false,
            status,
            hl7_message_sent: false,
            hl7_message_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: Uuid::new_v4(),
        }
    }

    fn test_result(flag: Option<ResultFlag>) -> PathologyResult {
        PathologyResult {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            order_id: Some(Uuid::new_v4()),
            laboratory: Laboratory::Sonic,
            lab_report_number: "LAB-2026-1".to_string(),
            collection_date: NaiveDate::from_ymd_opt(2026, 3, 1).expect("valid date"),
            report_date: Utc::now(),
            tests: vec![TestResult {
                test_name: "Haemoglobin".to_string(),
                test_code: Some("HB".to_string()),
                loinc_code: None,
                value: "90".to_string(),
                unit: Some("g/L".to_string()),
                reference_range: Some("115-165".to_string()),
                flag,
                status: ResultStatus::Final,
                comment: None,
            }],
            clinical_notes: None,
            pathologist_comment: None,
            has_abnormal: false,
            has_critical: false,
            acknowledged: false,
            acknowledged_by: None,
            acknowledged_at: None,
            hl7_message_received: true,
            hl7_message_id: Some("HL7-1".to_string()),
            pdf_report_path: None,
            received_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_create_order_rejects_empty_test_list() {
        let service = new_service(vec![]);
        let mut order = test_order(OrderStatus::Draft);
        order.tests.clear();

        let result = service.create_order(order).await;

        assert!(matches!(
            result,
            Err(ServiceError::Validation(ValidationError::EmptyTestList))
        ));
    }

    #[tokio::test]
    async fn test_create_result_sets_abnormal_flags() {
        let service = new_service(vec![]);
        let result = service
            .create_result(test_result(Some(ResultFlag::CriticalHigh)))
            .await;

        assert!(result.is_ok());
        let saved = result.expect("result should be ok");
        assert!(saved.has_abnormal);
        assert!(saved.has_critical);
    }

    #[tokio::test]
    async fn test_find_orders_by_status_returns_only_matching() {
        let service = new_service(vec![
            test_order(OrderStatus::Draft),
            test_order(OrderStatus::Ordered),
        ]);

        let result = service.find_orders_by_status(OrderStatus::Ordered).await;

        assert!(result.is_ok());
        let orders = result.expect("result should be ok");
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].status, OrderStatus::Ordered);
    }
}
