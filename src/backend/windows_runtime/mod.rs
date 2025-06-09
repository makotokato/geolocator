use crate::{GeolocatorCoordinates, GeolocatorError, GeolocatorOptions};
use windows::{Devices::Geolocation::*, Foundation::*};

impl From<windows_result::Error> for GeolocatorError {
    fn from(error: windows_result::Error) -> Self {
        match error.code() {
            windows_result::HRESULT(0) => panic!("Why S_OK"),
            _ => GeolocatorError::Unknown,
        }
    }
}

impl TryFrom<Geoposition> for GeolocatorCoordinates {
    type Error = windows_result::Error;

    fn try_from(geoposition: Geoposition) -> Result<Self, Self::Error> {
        let coordinate = geoposition.Coordinate()?;
        let point = coordinate.Point()?.Position()?;
        let latitude = point.Latitude;
        let longitude = point.Longitude;
        let altitude = Some(point.Altitude);
        let accuracy = coordinate.Accuracy()?;
        let altitude_accuracy = match coordinate.AltitudeAccuracy() {
            Ok(altitude_accuracy_ref) => altitude_accuracy_ref.Value().ok(),
            Err(_) => None,
        };
        let heading = match coordinate.Heading() {
            Ok(heading_ref) => heading_ref.Value().ok(),
            Err(_) => None,
        };
        let speed = match coordinate.Speed() {
            Ok(speed_ref) => speed_ref.Value().ok(),
            Err(_) => None,
        };

        Ok(Self {
            latitude,
            longitude,
            altitude,
            accuracy,
            altitude_accuracy,
            heading,
            speed,
        })
    }
}

pub(crate) struct GeolocatorImpl {
    geolocator: Geolocator,
    position_token: i64,
    status_token: i64,
}

impl GeolocatorImpl {
    pub fn new() -> Result<Self, GeolocatorError> {
        let geolocator = Geolocator::new()?;

        Ok(GeolocatorImpl {
            geolocator: geolocator,
            position_token: 0,
            status_token: 0,
        })
    }

    pub fn watch_position<
        F: Fn(GeolocatorCoordinates) + Send + Sync + 'static,
        E: Fn(GeolocatorError) + Send + Sync + 'static,
    >(
        &mut self,
        options: Option<GeolocatorOptions>,
        callback: F,
        error_callback: E,
    ) -> Result<(), GeolocatorError> {
        if let Some(options) = options {
            if options.high_accuracy {
                self.geolocator.SetDesiredAccuracy(PositionAccuracy::High)?;
            }
        }

        self.start_watch(callback, error_callback)
    }

    pub fn stop_watch(&mut self) -> Result<(), GeolocatorError> {
        self.geolocator.RemovePositionChanged(self.position_token)?;
        self.geolocator.RemoveStatusChanged(self.status_token)?;
        Ok(())
    }

    pub async fn current_position(&mut self) -> Result<GeolocatorCoordinates, GeolocatorError> {
        let geoposition = self.geolocator.GetGeopositionAsync()?.await?;
        let coordinates = geoposition.try_into()?;
        Ok(coordinates)
    }

    pub async fn request_access() -> Result<(), GeolocatorError> {
        let result = Geolocator::RequestAccessAsync()?.await?;
        match result {
            GeolocationAccessStatus::Allowed => Ok(()),
            _ => Err(GeolocatorError::AccessDenied),
        }
    }

    fn start_watch<
        F: Fn(GeolocatorCoordinates) + Send + Sync + 'static,
        E: Fn(GeolocatorError) + Send + Sync + 'static,
    >(
        &mut self,
        callback: F,
        error_callback: E,
    ) -> Result<(), GeolocatorError> {
        self.position_token = self.geolocator.PositionChanged(&TypedEventHandler::<
            Geolocator,
            PositionChangedEventArgs,
        >::new(
            move |_geolocator, event_args| {
                if let Some(event_args) = event_args.as_ref() {
                    let geoposition = event_args.Position()?;
                    let coordinate = geoposition.try_into()?;
                    callback(coordinate);
                }
                Ok(())
            },
        ))?;
        self.status_token = self.geolocator.StatusChanged(&TypedEventHandler::<
            Geolocator,
            StatusChangedEventArgs,
        >::new(
            move |_geolocator, event_args| {
                if let Some(event_args) = event_args.as_ref() {
                    match event_args.Status()? {
                        PositionStatus::Initializing | PositionStatus::Ready => {}
                        _ => error_callback(GeolocatorError::Unavailable),
                    };
                }
                Ok(())
            },
        ))?;

        Ok(())
    }
}
