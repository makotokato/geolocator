#[cfg(target_os = "windows")]
use crate::backend::GeolocatorImpl;

#[allow(unused)]
#[derive(Debug)]
pub enum GeolocatorError {
    AccessDenied,
    Unavailable,
    Unknown,
}

#[allow(unused)]
#[derive(Clone, Copy, Debug)]
pub struct GeolocatorCoordinates {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
    pub accuracy: f64,
    pub altitude_accuracy: Option<f64>,
    pub heading: Option<f64>,
    pub speed: Option<f64>,
}

pub struct GeolocatorOptions {
    pub high_accuracy: bool,
}

pub struct Geolocator {
    imp: GeolocatorImpl,
}

impl Geolocator {
    pub fn new() -> Result<Geolocator, GeolocatorError> {
        let imp = GeolocatorImpl::new()?;
        Ok(Geolocator { imp })
    }

    pub fn watch_position<
        F: Fn(GeolocatorCoordinates) + Send + Sync + 'static,
        E: Fn(GeolocatorError) + Send + Sync + 'static,
    >(
        &mut self,
        options: Option<GeolocatorOptions>,
        callback: F,
        error: E,
    ) -> Result<(), GeolocatorError> {
        self.imp.watch_position(options, callback, error)
    }

    pub fn clear_watch(&mut self) -> Result<(), GeolocatorError> {
        self.imp.stop_watch()
    }

    pub async fn current_position(&mut self) -> Result<GeolocatorCoordinates, GeolocatorError> {
        self.imp.current_position().await
    }

    pub async fn request_access() -> Result<(), GeolocatorError> {
        GeolocatorImpl::request_access().await
    }
}
