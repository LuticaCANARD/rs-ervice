# rs-ervice

- Service Singleton manager library for Rust.


## What is Service in this project


## Our goals

- Our final user's example is like this

```rust

#[singleton_service_struct]
struct MyService {
    pub state: String,
    //...
}

#[singleton_service]
impl MyService {
    pub fn new() -> Self {}

    pub async fn doing_something(&self, something: String) -> String {
        self.state.clone() + &something // self.state로 변경, String 연산 방식 수정
    }
 
}

/// on use...

use ...::{RSContextBuilder}
fn main(){

    let service_context = RSContextBuilder::new()
        .register::<MyService>() // 매크로가 MyService::new를 알 수 있도록 도와줌
        .register::<AnotherService>()
        .build();

    let do_service : String = service_context.call::<MyService>()
        .unwrap()
        .doing_something("Hi!".to_string())
        .await; // .await 추가
    

}


```
