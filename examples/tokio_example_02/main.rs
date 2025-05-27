#[cfg(feature = "tokio")]
mod services;
#[cfg(feature = "tokio")]
#[tokio::main]
async fn main() {

}

#[cfg(not(feature = "tokio"))]
fn main() {
    println!("This example requires the 'tokio' feature to run.");
    println!("Please run with 'cargo run --features tokio'.");
    std::process::exit(1);
}
