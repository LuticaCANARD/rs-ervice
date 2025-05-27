use std::any::{Any, TypeId};
use std::collections::BTreeMap;
use std::fmt; // For custom error
use std::sync::Arc;
use std::time::SystemTime; // 시간 정보 기록을 위해

// 각 서비스 항목에 대한 메타데이터
#[derive(Debug, Clone)] // 쉽게 확인하고 복사할 수 있도록
pub struct RsServiceEntryMetadata {
    pub type_id_repr: String,    // TypeId의 Debug 표현 (문자열)
    pub type_name: &'static str, // 서비스의 실제 타입 이름
    // 필요하다면 여기에 더 많은 정보 추가 가능 (예: 서비스 버전, 설명 등)
}
// 전체 RSContext에 대한 메타데이터
#[derive(Debug, Clone)]
pub struct RsContextMeta { // 이전 RsContextMetadata에서 이름 변경 제안
    pub registered_services: Vec<RsServiceEntryMetadata>,
    pub creation_timestamp: SystemTime, // 컨텍스트 생성 시각
}
struct StoredServiceInfo {
    container: ContainerStruct,    // Box<Arc<Mutex<T>>> 를 타입 소거한 것
    type_name: &'static str,
}
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
type MapForContainer = BTreeMap<TypeId, StoredServiceInfo>;

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
pub type AsyncHooksResult = Result<(), RsServiceError>;

#[cfg(feature = "tokio")]
pub trait RSContextService: Any + Send + Sync + 'static {
    /// Called by the framework to get a new instance of the service.
    /// Typically implemented by a procedural macro.
    async fn on_register_crate_instance() -> Self where Self: Sized;

    /// Called after the service instance is created and before it's wrapped
    /// in Arc<Mutex<T>> and stored in the builder.
    /// Ideal for initial setup that might need mutable access to self
    /// or access to builder configurations.
    fn on_service_created(&mut self, builder: &RSContextBuilder) -> impl std::future::Future<Output = AsyncHooksResult> + Send;

    /// (Optional) Called after all services are built and the RSContext is ready.
    /// This hook would be called on `&self` (obtained via MutexGuard).
    fn on_all_services_built(&self, context: &RSContext) -> impl std::future::Future<Output = AsyncHooksResult> + Send;
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
    pub async fn register<T>(mut self) -> Self
    where
        T: RSContextService + Send + Sync + 'static, // <- Send, Sync 추가
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
        let type_name_str: &'static str = std::any::type_name::<T>(); // Get the type name as a static string
        let service_info = StoredServiceInfo {
            container: Box::new(service_arc_mutex) as ContainerStruct,
            type_name: type_name_str,
        };
        self.pending_services.insert(
            type_id,service_info
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
                }) as Pin<Box<dyn Future<Output = Result<(), RsServiceError>> + Send>>
            });
            self.after_build_async_hooks.push(hook);
        }

        self
    }
    #[cfg(feature = "tokio")]
    /// Builds the RSContext from the registered services.
    pub async fn build(self) -> Result<RSContext, RsServiceError> { // Return Result for better error handling

        let mut metadata_entries: Vec<RsServiceEntryMetadata> = Vec::new();

        // 참조로 순회하여 메타데이터만 수집
        for (type_id, stored_info) in &self.pending_services {
            metadata_entries.push(RsServiceEntryMetadata {
                type_id_repr: format!("{:?}", type_id),
                type_name: stored_info.type_name,
            });
        }

        let context_metadata = RsContextMeta {
            registered_services: metadata_entries,
            creation_timestamp: SystemTime::now(),
        };
        let context = RSContext {
            service_map: self.pending_services,
            metadata: context_metadata
        };
        let arc_context = Arc::new(context);

        for async_hook in self.after_build_async_hooks {
            let fut = async_hook(Arc::clone(&arc_context)).await;
            fut.map_err(|e| RsServiceError(format!("Async after_build hook failed: {}", e)))?;
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
            panic!("Service type {:?} already registered.", std::any::type_name::<T>());
        }

        let mut instance = T::on_register_crate_instance();
        instance.on_service_created(&self)
            .map_err(|e| RsServiceError(format!("on_service_created hook failed for {}: {}", std::any::type_name::<T>(), e)))
            .expect("on_service_created hook failed");

        let service_arc_mutex: Arc<Mutex<T>> = Arc::new(Mutex::new(instance));
        let type_name_str: &'static str = std::any::type_name::<T>(); // 타입 이름 가져오기

        let service_info = StoredServiceInfo {
            container: Box::new(service_arc_mutex) as ContainerStruct,
            type_name: type_name_str,
        };
        self.pending_services.insert(type_id, service_info);

        // Register the after_build hook
        let hook = Box::new(move |ctx: &RSContext| {
            let arc_mutex = ctx.call::<T>().expect("Service not found");
            let guard = arc_mutex.lock().unwrap(); // MutexGuard 얻기
            guard.on_all_services_built(ctx)
        });
        self.after_build_hooks.push(hook);

        self
    }

    #[cfg(not(feature = "tokio"))]
    /// Builds the RSContext from the registered services.
    /// and calls the on_all_services_built hooks.
    pub fn build(self) -> Result<RSContext, RsServiceError> {
        let mut metadata_entries: Vec<RsServiceEntryMetadata> = Vec::new();

        // 참조로 순회하여 메타데이터만 수집
        for (type_id, stored_info) in &self.pending_services {
            metadata_entries.push(RsServiceEntryMetadata {
                type_id_repr: format!("{:?}", type_id),
                type_name: stored_info.type_name,
            });
        }

        let context_metadata = RsContextMeta {
            registered_services: metadata_entries,
            creation_timestamp: SystemTime::now(),
        };
        let context = RSContext {
            service_map: self.pending_services, // move는 여기서 한 번만!
            metadata: context_metadata,
        };

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
    pub metadata: RsContextMeta, // 공개 필드 또는 private + getter 메소드

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
                boxed_val.container.downcast_ref::<Arc<Mutex<T>>>()
            })
            .cloned()
    }

    pub fn is_registered<T: RSContextService>(&self) -> bool {
        self.service_map.contains_key(&TypeId::of::<T>())
    }
    pub fn get_metadata(&self) -> &RsContextMeta {
        &self.metadata
    }
}
