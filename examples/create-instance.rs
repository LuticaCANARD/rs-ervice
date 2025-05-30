use std::any::Any;
use rs_ervice::common::RsServiceError;
#[cfg(feature = "tokio")]
use rs_ervice::tokio_rs_ervice::{AsyncHooksResult,RSContextService,RSContextBuilder};

#[cfg(not(feature = "tokio"))]
use rs_ervice::{vanilla_rs_ervice::{RSContextBuilder, RSContextService}};
use rs_ervice_macro_lib::{r_service, r_service_struct};
#[r_service_struct]
#[derive(Debug, Clone)]
struct MyService {
    pub state: String,
    //...
}

trait Chant{
    fn chanting(st:String) -> String;
}

#[r_service]
impl MyService  {
    pub fn new() -> Self {
        MyService{
            state : "HELLO - ".to_string(),
        }
    }
    #[cfg(feature = "tokio")]
    pub async fn doing_something(&self, something: String) -> String {
        self.state.clone() + &something
    }
    #[cfg(not(feature = "tokio"))]
    pub fn doing_something(&self, something: String) -> String {
        self.state.clone() + &something
    }
}


impl Chant for MyService {
    fn chanting(st: String) -> String {
        st + "!!!!!"
    }
}

#[cfg(not(feature = "tokio"))]
impl RSContextService for MyService {
    fn on_register_crate_instance() -> Self {
        MyService::new()
    }
    
    fn on_service_created(&mut self, service_builder: &RSContextBuilder) -> Result<(), RsServiceError> {
        // 서비스가 등록될 때 호출되는 메서드
        println!("Service {} registered successfully!", std::any::type_name::<Self>());
        Ok(())
    }
    fn on_all_services_built(&self, context: &rs_ervice::RSContext) -> Result<(), RsServiceError> {
        // 모든 서비스가 빌드된 후 호출되는 메서드
        println!("All services built successfully in context: {:?}", context.type_id());
        Ok(())
    }
}

#[cfg(feature = "tokio")]
impl RSContextService for MyService {
    async fn on_register_crate_instance() -> Self {
        MyService::new()
    }
    
    async fn on_service_created(&mut self, service_builder: &RSContextBuilder) -> Result<(), RsServiceError> {
        // 서비스가 등록될 때 호출되는 메서드
        println!("Service {} registered successfully!", std::any::type_name::<Self>());
        Ok(())
    }
    async fn on_all_services_built(&self, context: &rs_ervice::RSContext) -> Result<(), RsServiceError> {
        // 모든 서비스가 빌드된 후 호출되는 메서드
        println!("All services built successfully in context: {:?}", context.type_id());
        Ok(())
    }
}

#[r_service_struct]
#[derive(Debug, Clone)]
struct AnotherService {

}

#[r_service]
impl AnotherService {
    pub fn new() -> Self {
        AnotherService {}
    }
}
#[cfg(not(feature = "tokio"))]
impl RSContextService for AnotherService {
    fn on_register_crate_instance() -> Self {
        AnotherService::new()
    }
    
    fn on_service_created(&mut self, service_builder: &RSContextBuilder) -> Result<(), RsServiceError> {
        print!("AnotherService registered!\n");
        Ok(())
    }
    fn on_all_services_built(&self, context: &rs_ervice::RSContext) -> Result<(), RsServiceError> {
        // 모든 서비스가 빌드된 후 호출되는 메서드
        println!("All services built successfully in context: {:?}", context.type_id());
        Ok(())
    }
}
#[cfg(feature = "tokio")]
impl RSContextService for AnotherService {
    async fn on_register_crate_instance() -> Self {
        AnotherService::new()
    }
    
    async fn on_service_created(&mut self, _service_builder: &RSContextBuilder) -> Result<(), RsServiceError> {
        print!("AnotherService registered!\n");
        Ok(())
    }
    async fn on_all_services_built(&self, context: &rs_ervice::RSContext)->AsyncHooksResult {
        // 모든 서비스가 빌드된 후 호출되는 메서드
        println!("All services built successfully in context: {:?}", context.type_id());
        Ok(())
    }
}

/// on use...



#[cfg(not(feature = "tokio"))]
fn main(){
    use rs_ervice::RSContext;


    fn build_context() -> Result<RSContext, RsServiceError> {
        Ok(
            RSContextBuilder::new()
                .register::<MyService>()?
                .register::<AnotherService>()?
                .build()
                .expect("Failed to build RSContext")
            )
    }
    let service_context = build_context().expect("Failed to create RSContext");
    let do_service = service_context.call::<MyService>();

    if do_service.is_none() {
        panic!("MyService is not registered!");
    }
    let do_service = do_service.unwrap().lock().unwrap().doing_something("Hi!".to_string());


    
    // Expected output: HELLO - Hi!
    println!("{}", do_service);
    
    // To call the Chanting method from the Chant trait implemented by MyService:
    // Since Chanting does not take &self, it's called like a static method associated with the trait implementation.
    let chanting_output = <MyService as Chant>::chanting("Hi!".to_string());
    // Expected output: HELLO - Hi!!!!!!!!!! (Original was "Hi!!!!!!!", but it takes the result of do_something)
    println!("{}", chanting_output)

}

#[cfg(feature = "tokio")]
#[tokio::main]
async fn main() {
    use rs_ervice::RSContext;
    async fn build_context() -> Result<RSContext, RsServiceError> {
        Ok(
            RSContextBuilder::new()
                .register::<MyService>()
                .await?
                .register::<AnotherService>()
                .await?
                .build()
                .await?
            )
    }
    let service_context = build_context().await.expect("Failed to build RSContext");

    let do_service = service_context.call::<MyService>();

    if do_service.is_none() {
        panic!("MyService is not registered!");
    }
    let do_service = do_service.unwrap().lock().await.doing_something("Hi!".to_string()).await;

    // Expected output: HELLO - Hi!
    println!("{}", do_service);

    // To call the Chanting method from the Chant trait implemented by MyService:
    let chanting_output = <MyService as Chant>::chanting("Hi!".to_string());
    // Expected output: HELLO - Hi!!!!!!!!!! (Original was "Hi!!!!!!!", but it takes the result of do_something)
    println!("{}", chanting_output);
}
