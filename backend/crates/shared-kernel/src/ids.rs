use serde::{Deserialize, Serialize};
use std::fmt;
use utoipa::ToSchema;
use uuid::Uuid;

macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
        #[serde(transparent)]
        #[schema(value_type = String, format = "uuid")]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            pub fn from_uuid(id: Uuid) -> Self {
                Self(id)
            }

            pub fn into_inner(self) -> Uuid {
                self.0
            }

            pub fn as_uuid(&self) -> &Uuid {
                &self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl From<Uuid> for $name {
            fn from(value: Uuid) -> Self {
                Self(value)
            }
        }

        impl From<$name> for Uuid {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}

define_id!(OrganizationId);
define_id!(UserId);
define_id!(FarmId);
define_id!(EnclosureId);
define_id!(SpeciesId);
define_id!(ProfileId);
define_id!(BatchId);
define_id!(IndividualTagId);
define_id!(WarehouseId);
define_id!(FeedProductId);
define_id!(DeviceId);
define_id!(OperationId);
