use std::{any::TypeId, sync::{Arc}};

use common::{CategoryType, MapForContainer};

pub mod common;

#[cfg(not(feature = "tokio"))]
pub mod vanilla_rs_ervice;
#[cfg(not(feature = "tokio"))]
use std::sync::Mutex;
#[cfg(feature = "tokio")]
use tokio::sync::Mutex;
#[cfg(not(feature = "tokio"))]
use vanilla_rs_ervice::RSContextService;
#[cfg(feature = "tokio")]
pub mod tokio_rs_ervice;
#[cfg(feature = "tokio")]
use tokio_rs_ervice::RSContextService;



// --- Core Service Trait ---
/// RSContextService: Trait for services that can be registered in RSContext.
/// This trait defines the lifecycle hooks for services in the context.
/// It will be managed by the RSContextBuilder.


// --- RSContext: Holds and provides access to services ---
/// The main context for managing registered services.
/// It provides methods to retrieve service instances.
pub struct RSContext where
{
        /// Stores Box<Arc<Mutex<T>>> type-erased as Box<dyn Any + ...>
    service_map: MapForContainer,
    category: CategoryType,
}

impl RSContext
    {
    /// Retrieves a shared, mutex-guarded service instance.
    /// Cloning the Arc increments the reference count, allowing shared ownership.
    pub fn call<T>(&self) -> Option<Arc<Mutex<T>>>
    where
        T: RSContextService, // T must be a registered service type
    {
        self.service_map
            .get(&TypeId::of::<T>())
            .and_then(|boxed_val| {
                boxed_val.downcast_ref::<Arc<Mutex<T>>>()
            })
            .cloned()
    }
}
