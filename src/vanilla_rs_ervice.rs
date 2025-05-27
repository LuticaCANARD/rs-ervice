// --- Conditional Mutex and Arc ---
use std::sync::Mutex;
use std::any::{Any, TypeId};
use std::collections::BTreeMap;
use std::sync::Arc;
use crate::common::{RsServiceError,MapForContainer, ContainerStruct};
use crate::RSContext;

pub trait RSContextService: Any + Send + Sync + 'static {
    /// Called by the framework to get a new instance of the service.
    /// Typically implemented by a procedural macro.
    fn on_register_crate_instance() -> Self where Self: Sized;

    /// Called after the service instance is created and before it's wrapped
    /// in Arc<Mutex<T>> and stored in the builder.
    /// Ideal for initial setup that might need mutable access to self
    /// or access to builder configurations.
    fn on_service_created(&mut self, builder: &RSContextBuilder) -> Result<(), RsServiceError>;

    /// (Optional) Called after all services are built and the RSContext is ready.
    /// This hook would be called on `&self` (obtained via MutexGuard).
    fn on_all_services_built(&self, context: &RSContext) -> Result<(), RsServiceError>;

}

type AfterBuildHook = Box<
    dyn FnOnce(&RSContext) -> 
        Result<(), RsServiceError> 
        + Send 
        + Sync
>;

// --- RSContextBuilder: For registering and building the context ---
#[cfg(not(feature = "tokio"))]
/// RSContextBuilder: For registering and building the context in non-tokio environments
pub struct RSContextBuilder {
    /// Stores Box<Arc<Mutex<T>>> type-erased as Box<dyn Any + ...>
    pending_services: MapForContainer,
    /// Stores closures to run after RSContext is built.
    after_build_hooks: Vec<AfterBuildHook>,
}
impl RSContextBuilder {

     #[cfg(not(feature = "tokio"))]
    pub fn new() -> Self {
        RSContextBuilder {
            pending_services: BTreeMap::new(),
            after_build_hooks: Vec::new(),
        }
    }
    #[cfg(not(feature = "tokio"))]
    /// Registers a service type T with the builder.
    /// T must implement RSContextService.
    pub fn register<T>(mut self) -> Result<Self,RsServiceError>
    where
        T: RSContextService, // T must implement RSContextService
    {
        

        let type_id = TypeId::of::<T>();
        if self.pending_services.contains_key(&type_id) {
            return Err(RsServiceError(format!("Service type {:?} already registered.", std::any::type_name::<T>())));
        }
        let mut instance = T::on_register_crate_instance();
        let result_on = instance.on_service_created(&self)
        .map_err(
            |e| 
            RsServiceError(format!("on_service_created hook failed for {}: {}", std::any::type_name::<T>(), e)
        ));
        if let Err(e) = result_on {
            return Err(e);
        }
        let service_arc_mutex: Arc<Mutex<T>> = Arc::new(Mutex::new(instance));

        // Store the Arc<Mutex<T>> itself, but boxed and type-erased.
        self.pending_services.insert(
            type_id,
            Box::new(service_arc_mutex.clone()) as ContainerStruct,
        );
        
        // Example: Preparing an after_build hook for this service T
        // This specific hook implementation would require T to implement on_all_services_built
        self.after_build_hooks.push(Box::new(move |ctx: &RSContext| {
            if let Some(service_access) = ctx.call::<T>() { // Using call to get the Arc<Mutex<T>>
                let service_guard = service_access.lock().map_err(|_| RsServiceError("Mutex poisoned".to_string()))?;
                service_guard.on_all_services_built(ctx)?;
            }
            Ok(())
        }));

        Ok(self)
    }

    #[cfg(not(feature = "tokio"))]
    /// Builds the RSContext from the registered services.
    /// and calls the on_all_services_built hooks.
    pub fn build(self) -> Result<RSContext, RsServiceError> { // Return Result for better error handling
        let context = RSContext {
            service_map: self.pending_services, // Move the map
        };

        // Call after_build hooks
        for hook_fn in self.after_build_hooks {
            hook_fn(&context)?;
        }

        Ok(context)
    }
}