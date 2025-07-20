//-----------------------------------
// Note: For tokio, locking is async. Lifecycle hooks and call patterns would need to be async.
// For this summary, we'll focus on the std::sync::Mutex path for simplicity in sync hook calls.
// A full tokio version would require async traits for hooks or async closures.
use tokio::sync::Mutex;
use std::{
    any::{Any, TypeId}, collections::BTreeMap, future::Future, pin::Pin, sync::Arc
};

use crate::{common::{ContainerStruct, MapForContainer, MapForContainerWithType, RsServiceError}, RSContext};
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
    category_info:Box<dyn Any + Send + Sync + 'static>,
    trait_relation: BTreeMap<TypeId, Box<MapForContainer>>
}


/// RSContextBuilder: For registering and building the context
impl RSContextBuilder {
    /// Creates a new RSContextBuilder instance.
    pub fn new() -> Self {
        RSContextBuilder {
            pending_services: BTreeMap::new(),
            after_build_async_hooks: Vec::new(),
            category_info: Box::new(()), // Placeholder for category info, can be replaced with actual type
            trait_relation: BTreeMap::new(),
        }
    }
    /// Registers a service type T with the builder.
    /// T must implement RSContextService.
    pub async fn register<T>(&mut self) -> Result<&mut Self,RsServiceError>
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

    pub fn add_relation_trait<Origin, Target>(&mut self) -> &mut Self
    where
        Origin: Any,
        Target: Any + Send + Sync + 'static, // 반드시 'static 필요
    {
        let origin_id = TypeId::of::<Origin>();
        let target_id = TypeId::of::<Target>();

        if !self.trait_relation.contains_key(&origin_id) {
            self.trait_relation.insert(origin_id, Box::new(BTreeMap::new()));
        }

        if let Some(target_map) = self.trait_relation.get_mut(&origin_id) {
            let instance = self.pending_services.get(&target_id);
            if instance.is_none() {
                println!(
                    "Warning: Target type {:?} is not registered.",
                    std::any::type_name::<Target>()
                );
                return self;
            }
            let instance = instance.unwrap();
            // 구체 타입으로 저장된 Arc<Mutex<Target>>를 트레잇 객체로 변환
            if let Some(concrete_arc) = instance.downcast_ref::<Origin>() {
                // Arc<Mutex<Target>> -> Arc<Mutex<dyn Origin + Send + Sync>>

                print!("Adding relation from {:?} to {:?}\n", 
                    std::any::type_name::<Origin>(), 
                    std::any::type_name::<Target>()
                );
            } else {
                println!("Failed to downcast to Arc<Mutex<Target>> for trait relation.");
            }
        }
        self
    }
    /// Builds the RSContext from the registered services.
    pub fn set_category<TC>(mut self, _category: TC) -> Result<Self, RsServiceError>
    where
        TC: Any + Send + Sync + 'static, // Ensure TC is a type that can be boxed
    {
        self.category_info = Box::new(_category);
        Ok(self)
    }
    pub async fn build(&mut self) -> Result<RSContext, RsServiceError> { // Return Result for better error handling
        let category = std::mem::replace(&mut self.category_info, Box::new(()));
        let service_map = std::mem::replace(&mut self.pending_services, BTreeMap::new());
        let service_map_for_trait = std::mem::replace(&mut self.trait_relation, BTreeMap::new());

        let context = RSContext {
            category,
            service_map,
            service_map_for_trait,
            ..Default::default()
        };
        let arc_context = Arc::new(context);

        for async_hook in &self.after_build_async_hooks {
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