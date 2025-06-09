use geolocator::*;

#[tokio::main]
async fn main() {
    let mut provider = Geolocator::new().expect("create Geolocator");
    Geolocator::request_access().await.expect("allow request");
    let coordinates = provider.current_position().await;
    println!("{:?}", coordinates.expect("get coordinate"));
}
