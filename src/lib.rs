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
    service_map_for_trait: BTreeMap<TypeId, Box<MapForContainer>>,

}
impl Default for RSContext {
    fn default() -> Self {
        RSContext {
            service_map: BTreeMap::new(),
            category: Box::new(()),
            service_map_for_trait: BTreeMap::new(),
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


    #[cfg(not(feature = "tokio"))]
    /// Retrieves a shared, mutex-guarded service instance, with lock
    pub fn call_with_lock<T>(
        &self,
    ) -> Option<std::sync::MutexGuard<'_, T>>
    where
        T: RSContextService,
    {
        self.service_map
            .get(&TypeId::of::<T>())
            .and_then(|boxed_val| {
                boxed_val.downcast_ref::<Arc<Mutex<T>>>()
            })
            .and_then(|arc_mutex| arc_mutex.lock().ok())
    }
    #[cfg(feature = "tokio")]
    /// Retrieves a shared, mutex-guarded service instance, with lock
    pub async fn call_with_lock<T>(
        &self,
    ) -> Option<tokio::sync::MutexGuard<'_, T>>
    where
        T: RSContextService,
    {
        let ret = self.service_map
            .get(&TypeId::of::<T>())
            .and_then(|boxed_val| {
                boxed_val.downcast_ref::<Arc<Mutex<T>>>()
            });
        if let Some(arc_mutex) = ret {
            Some(arc_mutex.lock().await);
        }
        None
    }
    

    #[cfg(not(feature = "tokio"))]
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
    #[cfg(feature = "tokio")]
    pub async fn call_services_by_trait<T>(
        &mut self,
    ) -> Option<Vec<Arc<Mutex<T>>>>
    where
        T: Any,
    {
        let trait_id = TypeId::of::<T>();
        println!("Searching for cached services for trait {:?}", std::any::type_name::<T>());
        println!("Searching for cached services for trait {:?}", trait_id);

        println!("Trait ID: {:?}", self.service_map_for_trait.keys());
        if let Some(services) = self.service_map_for_trait.get(&trait_id) {
            println!("Found cached services for trait {:?}", std::any::type_name::<T>());
            println!("Cached services: {:?}", services.values());
            // 트레잇 객체로 다운캐스팅
            let result: Vec<Arc<Mutex<T>>> = services.values()
                .filter_map(|boxed_val| {
                    // Arc<Mutex<dyn Trait>>로 다운캐스팅
                    boxed_val.downcast_ref::<Arc<Mutex<T>>>().cloned()
                })
                .collect();
            if !result.is_empty() {
                return Some(result);
            }
        }
        None
    }
}
