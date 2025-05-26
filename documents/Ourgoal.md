# Our goals

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