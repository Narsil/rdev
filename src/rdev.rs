use serde::ser::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, PartialEq)]
pub enum EventError {
    InvalidCoordinates,
    InvalidName,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EventType {
    KeyPress { code: u8 },
    KeyRelease { code: u8 },
    ButtonPress { code: u8 },
    ButtonRelease { code: u8 },
    MouseMove { x: f64, y: f64 },
    Wheel { delta_x: f64, delta_y: f64 },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    #[serde(
        serialize_with = "serialize_from_st",
        deserialize_with = "deserialize_from_ts"
    )]
    time: SystemTime,
    #[serde(default)]
    name: Option<String>,
    event_type: EventType,
}

impl Event {
    pub fn new(
        event_type: EventType,
        time: SystemTime,
        name: Option<String>,
    ) -> Result<Event, EventError> {
        if let EventType::MouseMove { x, y } = event_type {
            if x < 0.0 || x > 1.0 || y < 0.0 || y > 1.0 {
                return Err(EventError::InvalidCoordinates);
            }
        }
        if let Some(event_name) = &name {
            if (event_name.contains(' ') && event_name != " ") || event_name == "" {
                return Err(EventError::InvalidName);
            }
        }
        Ok(Event {
            event_type,
            time,
            name,
        })
    }
}

fn serialize_from_st<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match time.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => serializer.serialize_f64(n.as_secs_f64()),
        Err(e) => Err(e).map_err(Error::custom),
    }
}
fn deserialize_from_ts<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
where
    D: Deserializer<'de>,
{
    match f64::deserialize(deserializer) {
        Ok(v) => {
            println!("Deserializing {}", v);
            let d = Duration::from_secs_f64(v);
            println!("Deserializing {:?}", d);
            let res = SystemTime::UNIX_EPOCH + d;
            println!("Result {:?}", res);
            Ok(res)
        }
        Err(e) => Err(e),
    }
}

pub fn format_events(events: &[Event]) -> String {
    let mut s = String::from("");
    for event in events {
        if let Some(event_name) = &event.name {
            match event.event_type {
                EventType::KeyPress { .. } => s.push_str(&event_name),
                EventType::ButtonPress { .. } => s.push_str(&event_name),
                _ => (),
            }
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use crate::rdev::{format_events, Event, EventError, EventType};
    use std::time::SystemTime;

    fn simple_key(code: u8, name: Option<String>) -> Event {
        Event::new(EventType::KeyPress { code }, SystemTime::now(), name).unwrap()
    }

    #[test]
    fn format_simple() {
        let event = simple_key(1, Some(String::from("H")));
        let event2 = simple_key(2, Some(String::from("e")));
        assert_eq!(format_events(&vec![event, event2]), String::from("He"));
    }

    #[test]
    fn event_out_of_boundary() {
        let event = Event::new(
            EventType::MouseMove { x: -1.0, y: 0.0 },
            SystemTime::now(),
            Some(String::from("e")),
        );
        assert_eq!(
            event.err(),
            Some(EventError::InvalidCoordinates),
            "We should crash because x is negative"
        );
    }

    #[test]
    fn event_invalid_name() {
        let event = Event::new(
            EventType::KeyPress { code: 1 },
            SystemTime::now(),
            Some(String::from("")),
        );
        assert_eq!(
            event.err(),
            Some(EventError::InvalidName),
            "We should not emit empty string names"
        );
    }

    #[test]
    fn event_invalid_name_with_space() {
        let event = Event::new(
            EventType::KeyPress { code: 1 },
            SystemTime::now(),
            Some(String::from("Some event")),
        );
        assert_eq!(
            event.err(),
            Some(EventError::InvalidName),
            "We should not emit an event associated with a space"
        );
    }

    // #[test]
    // fn event_json_deserialization() {
    //     let data = r#"
    //     {
    //         "keycode": 1,
    //         "hasAlt": false,
    //         "hasCapsLock": false,
    //         "hasCtrl": false,
    //         "hasMeta": false,
    //         "hasShift": false,
    //         "time": 15806656262720,
    //         "press": true,
    //         "event_type": "KEY",
    //         "x":0.0,
    //         "y":0.0
    //     }"#;

    //     // Parse the string of data into serde_json::Value.
    //     let event: Event = serde_json::from_str(data).unwrap();
    //     if let EventType::KeyPress { code } = event.event_type {
    //         assert_eq!(code, 1);
    //     } else {
    //         panic!("Incorrect type");
    //     }
    // }

    // #[test]
    // fn event_json_serialization() {
    //     let event = Event::new(
    //         EventType::KeyPress { code: 1 },
    //         SystemTime::UNIX_EPOCH,
    //         None,
    //     )
    //     .unwrap();
    //     let data = r#"{"keycode":1,"hasAlt":false,"hasCapsLock":false,"hasCtrl":false,"hasMeta":false,"hasShift":false,"time":0.0,"press":true,"event_type":"KEY","x":0.0,"y":0.0}"#;

    //     // Parse the string of data into serde_json::Value.
    //     let output: String = serde_json::to_string(&event).unwrap();
    //     assert_eq!(output, data);
    // }
}
