//-----------------------------------
// Note: For tokio, locking is async. Lifecycle hooks and call patterns would need to be async.
// For this summary, we'll focus on the std::sync::Mutex path for simplicity in sync hook calls.
// A full tokio version would require async traits for hooks or async closures.
use tokio::sync::Mutex;
use std::{
    any::{Any, TypeId}, collections::BTreeMap, future::Future, pin::Pin, sync::Arc
};

use crate::{common::{ContainerStruct, MapForContainer, RsServiceError}, RSContext};
pub type AsyncHooksResult = Result<(), RsServiceError>;
pub trait RSContextService: Any {
    /// Called by the framework to get a new instance of the service.
    /// Typically implemented by a procedural macro.
    fn on_register_crate_instance() -> impl Future<Output=Self> where Self: Sized;

    /// Called after the service instance is created and before it's wrapped
    /// in Arc<Mutex<T>> and stored in the builder.
    /// Ideal for initial setup that might need mutable access to self
    /// or access to builder configurations.
    fn on_service_created(&mut self, builder: &RSContextBuilder) -> impl std::future::Future<Output = AsyncHooksResult>;

    /// (Optional) Called after all services are built and the RSContext is ready.
    /// This hook would be called on `&self` (obtained via MutexGuard).
    fn on_all_services_built(&self, context: &RSContext) -> impl std::future::Future<Output = AsyncHooksResult>;
}
type FutureHookResult = Pin<Box<dyn Future<Output = AsyncHooksResult>>>;
type AfterAsyncBuildHook = Box<
    dyn Fn(Arc<RSContext>) -> FutureHookResult
>;

/// RSContextBuilder: For registering and building the context in tokio
pub struct RSContextBuilder {
    pending_services: MapForContainer,
    after_build_async_hooks: Vec<AfterAsyncBuildHook>,
    category_info:Box<dyn Any + Send + Sync + 'static>
}


/// RSContextBuilder: For registering and building the context
impl RSContextBuilder {
    /// Creates a new RSContextBuilder instance.
    pub fn new() -> Self {
        RSContextBuilder {
            pending_services: BTreeMap::new(),
            after_build_async_hooks: Vec::new(),
            category_info: Box::new(()), // Placeholder for category info, can be replaced with actual type
        }
    }
    /// Registers a service type T with the builder.
    /// T must implement RSContextService.
    pub async fn register<T>(mut self) -> Result<Self,RsServiceError>
    where
        T: RSContextService, // <- Send, Sync 추가
    {
        let type_id = TypeId::of::<T>();
        if self.pending_services.contains_key(&type_id) {
            return Err(RsServiceError(format!("Service type {:?} already registered.", std::any::type_name::<T>())));
        }

        let mut instance = T::on_register_crate_instance().await;

        instance.on_service_created(&self)
            .await
            .map_err(
                |e| RsServiceError(format!("on_service_created hook failed for {}: {}", std::any::type_name::<T>(), e))
            )?;

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
                    arc_mutex.lock().await.on_all_services_built(&ctx).await
                }) as FutureHookResult
            });
            self.after_build_async_hooks.push(hook);
        }

        Ok(self)
    }
    /// Builds the RSContext from the registered services.
    pub fn set_category<TC>(mut self, _category: TC) -> Result<Self, RsServiceError>
    where
        TC: Any + Send + Sync + 'static, // Ensure TC is a type that can be boxed
    {
        self.category_info = Box::new(_category);
        Ok(self)
    }
    pub async fn build(self) -> Result<RSContext, RsServiceError> { // Return Result for better error handling
        let context = RSContext {
            category: self.category_info,
            service_map: self.pending_services,
            ..Default::default()
        };
        let arc_context = Arc::new(context);

        for async_hook in self.after_build_async_hooks {
            let fut = async_hook(Arc::clone(&arc_context)).await;
            if let Err(e) = fut {
                return Err(e);
            }
        }

        match Arc::try_unwrap(arc_context) {
            Ok(context) => Ok(context),
            Err(_) => Err(RsServiceError("Failed to unwrap Arc<RSContext> in build()".to_string())),
        }
    }
}