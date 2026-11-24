pub mod error;
pub mod extractor;
pub mod jwt;
pub mod rbac;
pub mod sync;

pub use error::ApiError;
pub use extractor::{AppBackendState, AuthenticatedUser};
pub use jwt::{JwtClaims, JwtConfig, JwtService};
pub use rbac::{verify_membership_and_permissions, AuthPrincipal, Permission};
pub use sync::{
    validate_conflict_resolution, ConflictResolution, OutboxChangeEvent, RegisterDeviceRequest,
    RegisterDeviceResponse, SyncCommand, SyncConflictRecord, SyncOperationType, SyncPullResponse,
    SyncPushItemResult, SyncPushRequest, SyncPushResponse, SyncService, SyncStatus,
};
