# rs-ervice

**intuitive & powerful library Service manager.**

- Service manager library for Rust.

## What is **Service** in this project

- In `rs-ervice`, a `Service` is a Rust struct that bundles related state with the methods (actions) that operate on that state.
- These services are responsible for a specific domain of logic or functionality within an application, such as handling user authentication or managing data access.
- They are registered with and managed by an `RSContext`, which provides them as singleton instances for consistent and easy access.

## Our goals

- Our final user's example is like this

```rust

#[r_service_struct]
struct MyService {
    pub state: String,
    //...
}

trait Chant{
    pub fn Chanting(st:String) -> String;
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
    fn Chanting(st: String) -> String {
        st + "!!!!!"
    }
}

#[r_service_struct]
struct AnotherService {

}

#[r_service]
impl AnotherService {
    pub fn new() -> Self {
        AnotherService {}
    }
}

/// on use...

use rs_ervice::{RSContextBuilder}; 
#[tokio::main] 
async fn main(){

    let service_context = RSContextBuilder::new()
        .register::<MyService>() // 매크로가 MyService::new를 알 수 있도록 도와줌
        .register::<AnotherService>()
        .build();

    let do_service : String = service_context.call::<MyService>()
        .unwrap()
        .doing_something("Hi!".to_string())
        .await;

    
    // Expected output: HELLO - Hi!
    println!("{}", do_service);
    
    // To call the Chanting method from the Chant trait implemented by MyService:
    // Since Chanting does not take &self, it's called like a static method associated with the trait implementation.
    let chanting_output = <MyService as Chant>::Chanting("Hi!".to_string());
    // Expected output: HELLO - Hi!!!!!!!!!! (Original was "Hi!!!!!!!", but it takes the result of do_something)
    println!("{}", chanting)

}


```

- our key word is `intuitive`.
- we hope make intuitive & powerful library Service manager.

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

> Not published now... so, clone this repo, and include by this

```toml
    [dependencies]
    rs-ervice = { path = "/path/to/cloned/rs-ervice" } 
```

## License

[MIT License](./License)

## If you want Contact

- @LuticaCANARD <presan100@gmail.com>
