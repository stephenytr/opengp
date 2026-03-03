//! Domain service macros
//!
//! This module provides macros to reduce boilerplate in domain service construction.
//! Instead of manually defining struct fields and constructors, services can use
//! the `service!` macro to generate both.
//!
//! # Example
//!
//! ```ignore
//! service! {
//!     PatientService {
//!         repository: Arc<dyn PatientRepository>,
//!     }
//! }
//! ```
//!
//! This expands to:
//!
//! ```ignore
//! pub struct PatientService {
//!     pub repository: Arc<dyn PatientRepository>,
//! }
//!
//! impl PatientService {
//!     pub fn new(repository: Arc<dyn PatientRepository>) -> Self {
//!         Self { repository }
//!     }
//! }
//! ```

/// Macro to generate a domain service struct and constructor
///
/// This macro generates:
/// - A public struct with the specified name and fields
/// - A `new()` constructor that takes the same fields and returns `Self`
///
/// # Fields
///
/// All fields are generated as public (`pub`) so they can be accessed by
/// other modules that need to use the service.
///
/// # Usage
///
/// ```ignore
/// service! {
///     MyService {
///         field1: Type1,
///         field2: Type2,
///         // ... more fields
///     }
/// }
/// ```
#[macro_export]
macro_rules! service {
    (
        $name:ident {
            $($field:ident: $type:ty),* $(,)?
        }
    ) => {
        pub struct $name {
            $(pub $field: $type),*
        }

        impl $name {
            pub fn new($($field: $type),*) -> Self {
                Self {
                    $($field),*
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    // Test trait for testing
    trait TestRepository: Send + Sync {}
    struct TestRepo;
    impl TestRepository for TestRepo {}

    // Test single field service
    service! {
        SingleFieldService {
            repository: Arc<dyn TestRepository>,
        }
    }

    // Test multi-field service
    service! {
        MultiFieldService {
            repository: Arc<dyn TestRepository>,
            audit_service: Arc<TestRepo>,
            query_service: Arc<TestRepo>,
        }
    }

    #[test]
    fn test_single_field_service() {
        let repo: Arc<dyn TestRepository> = Arc::new(TestRepo);
        let service = SingleFieldService::new(repo.clone());

        assert!(Arc::ptr_eq(&service.repository, &repo));
    }

    #[test]
    fn test_multi_field_service() {
        let repo: Arc<dyn TestRepository> = Arc::new(TestRepo);
        let audit: Arc<TestRepo> = Arc::new(TestRepo);
        let query: Arc<TestRepo> = Arc::new(TestRepo);

        let service = MultiFieldService::new(repo.clone(), audit.clone(), query.clone());

        assert!(Arc::ptr_eq(&service.repository, &repo));
        assert!(Arc::ptr_eq(&service.audit_service, &audit));
        assert!(Arc::ptr_eq(&service.query_service, &query));
    }
}
