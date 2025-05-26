# rs-ervice

**intuitive & powerful library Service manager.**

- Service manager library for Rust.

## What is **Service** in this project

- In `rs-ervice`, a `Service` is a Rust struct that bundles related state with the methods (actions) that operate on that state.
- These services are responsible for a specific domain of logic or functionality within an application, such as handling user authentication or managing data access.
- They are registered with and managed by an `RSContext`, which provides them as singleton instances for consistent and easy access.

## How to use

### if you use not tokio

```rust
use std::any::Any;

#[derive(Debug, Clone)]
struct MyService {
    pub state: String,
    //...
}

trait Chant{
    fn chanting(st:String) -> String;
}

impl MyService  {
    pub fn new() -> Self {
        MyService{
            state : "HELLO - ".to_string(),
        }
    }

    pub fn doing_something(&self, something: String) -> String {
        self.state.clone() + &something
    }
}

impl Chant for MyService {
    fn chanting(st: String) -> String {
        st + "!!!!!"
    }
}
impl RSContextService for MyService {
    /// this hook is call on registering
    fn on_register_crate_instance() -> Self {
        MyService::new()
    }
    
    /// this hook is call after register
    fn on_service_created(&mut self, service_builder: &RSContextBuilder) -> Result<(), RsServiceError> {
        println!("Service {} registered successfully!", std::any::type_name::<Self>());
        Ok(())
    }
    /// this hook is call after build 
    fn on_all_services_built(&self, context: &rs_ervice::RSContext) -> Result<(), RsServiceError> {
        
        println!("All services built successfully in context: {:?}", context.type_id());
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct AnotherService {

}

impl AnotherService {
    pub fn new() -> Self {
        AnotherService {}
    }
}

impl RSContextService for AnotherService {
    fn on_register_crate_instance() -> Self {
        AnotherService::new()
    }
    
    fn on_service_created(&mut self, service_builder: &RSContextBuilder) -> Result<(), RsServiceError> {
        print!("AnotherService registered!\n");
        Ok(())
    }
    fn on_all_services_built(&self, context: &rs_ervice::RSContext) -> Result<(), RsServiceError> {
        println!("All services built successfully in context: {:?}", context.type_id());
        Ok(())
    }
}

/// on use...

use rs_ervice::{RSContextBuilder, RSContextService, RsServiceError}; 

fn main(){

    let service_context = RSContextBuilder::new()
        .register::<MyService>()
        .register::<AnotherService>()
        .build()
        .expect("Failed to build RSContext");

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
```

### if you use tokio

```rust
use std::any::Any;

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

    pub async fn doing_something(&self, something: String) -> String {
        self.state.clone() + &something
    }
}

impl Chant for MyService {
    fn chanting(st: String) -> String {
        st + "!!!!!"
    }
}
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

#[tokio::main]
async fn main() {
    let service_context = RSContextBuilder::new()
        .register::<MyService>()
        .register::<AnotherService>()
        .build()
        .await
        .expect("Failed to build RSContext");

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


```

- Our Final implement goal is [here](./documents/Ourgoal.md)

### Features

- `Service Context` : Manages diverse service types within an isolated scope.

> This allows for clear separation of concerns in service management and enhances testability by providing distinct contexts.

- `Intuitive Macro System`: Define services effortlessly using `#[r_service_struct]` and `#[r_service]` attributes, significantly reducing boilerplate code.
- `Async Ready`: Designed with asynchronous operations in mind, allowing service methods to be async and integrate seamlessly.
- `Type-Safe Resolution`: Retrieve service instances with `call::<YourService>()`, ensuring type safety at compile time.
- `Composable Services`: Services managed by `rs-ervice` are standard Rust structs and can implement any number of traits, allowing for rich composition of behaviors and integration with other parts of your application or ecosystem. (Our example demonstrates this with the `Chant` trait).

## Contributing

- If you want contribute, first, fork this repository.
- then make a code & etc...
- after then, Pull Request to this repository. and wait for main contributor's review.

## Installation

> Just use this command!

`cargo add rs_ervice`

## License

[MIT License](./License)

## If you want Contact

- @LuticaCANARD <presan100@gmail.com>
