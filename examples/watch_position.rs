use geolocator::*;

#[tokio::main]
async fn main() {
    let mut locator = Geolocator::new().expect("create Geolocator");
    Geolocator::request_access().await.expect("allow request");

    let (tx, rx) = std::sync::mpsc::channel();
    let _ = locator.watch_position(None, move |coordinates| {
        tx.send(coordinates).expect("send should be successful")
    }, |_| {});

    let coordinates = rx.recv().expect("recv should be successful");
    println!("{:?}", coordinates);
    locator.clear_watch().expect("clear watch should be successful");
}