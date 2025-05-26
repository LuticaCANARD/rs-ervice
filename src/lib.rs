use std::any::{Any, TypeId};
use std::collections::BTreeMap;
use std::fmt; // For custom error
use std::sync::Arc;


// --- Conditional Mutex and Arc ---
#[cfg(not(feature = "tokio"))]
use std::sync::Mutex;
//-----------------------------------
// Note: For tokio, locking is async. Lifecycle hooks and call patterns would need to be async.
// For this summary, we'll focus on the std::sync::Mutex path for simplicity in sync hook calls.
// A full tokio version would require async traits for hooks or async closures.
#[cfg(feature = "tokio")]
use tokio::sync::Mutex;
#[cfg(feature = "tokio")]
use std::{
    pin::Pin,
    future::Future
};
// --- Custom Error Type (Example) ---
#[derive(Debug)]
pub struct RsServiceError(String);

impl fmt::Display for RsServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RsService Error: {}", self.0)
    }
}

impl std::error::Error for RsServiceError {

}
type ContainerStruct = Box<dyn Any + Send + Sync + 'static>;
type MapForContainer = BTreeMap<TypeId, ContainerStruct>;

// --- Core Service Trait ---
/// RSContextService: Trait for services that can be registered in RSContext.
/// This trait defines the lifecycle hooks for services in the context.
/// It will be managed by the RSContextBuilder.
#[cfg(not(feature = "tokio"))]
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
#[cfg(feature = "tokio")]
pub trait RSContextService: Any + Send + Sync + 'static {
    /// Called by the framework to get a new instance of the service.
    /// Typically implemented by a procedural macro.
    async fn on_register_crate_instance() -> Self where Self: Sized;

    /// Called after the service instance is created and before it's wrapped
    /// in Arc<Mutex<T>> and stored in the builder.
    /// Ideal for initial setup that might need mutable access to self
    /// or access to builder configurations.
    async fn on_service_created(&mut self, builder: &RSContextBuilder) -> Result<(), RsServiceError>;

    /// (Optional) Called after all services are built and the RSContext is ready.
    /// This hook would be called on `&self` (obtained via MutexGuard).
    async fn on_all_services_built(&self, context: &RSContext) -> Result<(), RsServiceError>;
}

#[cfg(not(feature = "tokio"))]
type AfterBuildHook = Box<
    dyn FnOnce(&RSContext) -> 
        Result<(), RsServiceError> 
        + Send 
        + Sync
>;
#[cfg(feature = "tokio")]
type AfterAsyncBuildHook = Box<
    dyn Fn(Arc<RSContext>) -> Pin<Box<dyn Future<Output = Result<(), RsServiceError>> + Send>>
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
#[cfg(feature = "tokio")]
/// RSContextBuilder: For registering and building the context in tokio
pub struct RSContextBuilder {
    pending_services: MapForContainer,
    after_build_async_hooks: Vec<AfterAsyncBuildHook>,
}
/// RSContextBuilder: For registering and building the context
impl RSContextBuilder {
    #[cfg(feature = "tokio")]
    /// Creates a new RSContextBuilder instance.
    pub fn new() -> Self {
        RSContextBuilder {
            pending_services: BTreeMap::new(),
            after_build_async_hooks: Vec::new(),
        }
    }
    #[cfg(feature = "tokio")]
    /// Registers a service type T with the builder.
    /// T must implement RSContextService.
    pub fn register<T>(mut self) -> Self
    where
        T: RSContextService, // T must implement RSContextService
    {
        let type_id = TypeId::of::<T>();
        if self.pending_services.contains_key(&type_id) {
            panic!("Service type {:?} already registered.", std::any::type_name::<T>());
        }

        let mut instance = T::on_register_crate_instance().await;

        instance.on_service_created(&self)
            .await
            .map_err(|e| RsServiceError(format!("on_service_created hook failed for {}: {}", std::any::type_name::<T>(), e)))
            .expect("on_service_created hook failed");

        let service_arc_mutex: Arc<Mutex<T>> = Arc::new(Mutex::new(instance));

        self.pending_services.insert(
            type_id,
            Box::new(service_arc_mutex.clone()) as ContainerStruct,
        );

        // Note: after_build_hooks must be async for tokio
        // You may want to define a separate Vec for async hooks, or use a feature flag.
        // For demonstration, let's assume you add an `after_build_async_hooks` Vec:
        // (You will need to add this field to RSContextBuilder for tokio)
        {
            let hook = Box::new(move |ctx: Arc<RSContext>| {
                let arc_mutex = ctx.call::<T>().expect("Service not found");
                Box::pin(async move {
                    let ret = arc_mutex.lock().await;
                    ret.on_all_services_built(&ctx).await;
                })  as Pin<Box<dyn Future<Output = Result<(), RsServiceError>> + Send>>
            });
            self.after_build_async_hooks.push(hook);
        }

        self
    }
    #[cfg(feature = "tokio")]
    /// Builds the RSContext from the registered services.
    pub async fn build(self) -> Result<RSContext, RsServiceError> { // Return Result for better error handling
        let context = RSContext {
            service_map: self.pending_services,
        };
        let arc_context = Arc::new(context);

        for async_hook in self.after_build_async_hooks {
            let fut = async_hook(Arc::clone(&arc_context)).await;
        }
        match Arc::try_unwrap(arc_context) {
            Ok(context) => Ok(context),
            Err(_) => Err(RsServiceError("Failed to unwrap Arc<RSContext> in build()".to_string())),
        }
    }
    /// You can use this method to register services in a non-tokio environment.
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
    pub fn register<T>(mut self) -> Self
    where
        T: RSContextService, // T must implement RSContextService
    {
        let type_id = TypeId::of::<T>();
        if self.pending_services.contains_key(&type_id) {
            // Or return Result<Self, Error>, or log. For now, panic.
            panic!("Service type {:?} already registered.", std::any::type_name::<T>());
        }

        let mut instance = T::on_register_crate_instance();

        // Call the on_service_created hook (receives &mut T and &RSContextBuilder)
        instance.on_service_created(&self)
            .map_err(|e| 
                RsServiceError(format!("on_service_created hook failed for {}: {}", std::any::type_name::<T>(), e))
            )
            .expect("on_service_created hook failed");

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

        self
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

// --- RSContext: Holds and provides access to services ---
/// The main context for managing registered services.
/// It provides methods to retrieve service instances.
pub struct RSContext {
    /// Stores Box<Arc<Mutex<T>>> type-erased as Box<dyn Any + ...>
    service_map: MapForContainer,
}

impl RSContext {
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
