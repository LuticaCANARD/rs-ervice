use std::{any::{Any, TypeId}, collections::BTreeMap, sync::Arc};

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
    cache_for_call_service: BTreeMap<TypeId, Arc<Mutex<dyn Any>>>,
}
impl Default for RSContext {
    fn default() -> Self {
        RSContext {
            service_map: BTreeMap::new(),
            category: Box::new(()),
            cache_for_call_service: BTreeMap::new(),
        }
    }
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

    pub fn call_services_by_trait<T>(
        &mut self,
    ) -> Option<Vec<Arc<Mutex<T>>>>
    where T:Any
    {
        let trait_id = TypeId::of::<T>();
        if let Some(services) = self.cache_for_call_service.get(&trait_id) {
            // If we have cached services, return them
            if let Ok(guard) = services.lock() {
                if let Some(vec) = guard.downcast_ref::<Vec<Arc<Mutex<T>>>>() {
                    return Some(vec.clone());
                }
            }
        }
        for (_, service) in &self.service_map {
            if let Some(vec) = service.downcast_ref::<Vec<Arc<Mutex<T>>>>() {
                // Cache the result for future calls
                self.cache_for_call_service.insert(
                    trait_id,
                    Arc::new(Mutex::new(vec.clone())),
                );
                return Some(vec.clone());
            }
        }
        None
    }
}
