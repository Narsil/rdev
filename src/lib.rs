
mod rdev{
    use serde::{Deserialize, Serialize, Serializer, Deserializer};
    use serde::ser::{Error};
    use std::time::{SystemTime, Duration};

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "UPPERCASE")]
    pub enum EventType{
        Key,
        Button,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum EventError{
        InvalidCoordinates,
        InvalidName,
        InvalidTime,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Event{
        #[serde(rename = "keycode")]
        pub code: u8,
        #[serde(rename = "hasAlt")]
        has_alt: bool,
        #[serde(rename = "hasCapsLock")]
        has_capslock: bool,
        #[serde(rename = "hasCtrl")]
        has_ctrl: bool,
        #[serde(rename = "hasMeta")]
        has_meta: bool,
        #[serde(rename = "hasShift")]
        has_shift: bool,
        #[serde(serialize_with = "serialize_from_st", deserialize_with="deserialize_from_ts")]
        time: SystemTime,
        press: bool,
        event_type: EventType,
        x: f64,
        y: f64,
        name: Option<String>,
    }

    fn serialize_from_st<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match time.duration_since(SystemTime::UNIX_EPOCH){
            Ok(n) => serializer.serialize_f64(n.as_secs_f64()),
            Err(e) => Err(e).map_err(Error::custom)
        }

    }
    fn deserialize_from_ts<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        match f64::deserialize(deserializer){
            Ok(v) => {
                println!("Deserializing {}", v);
                let d = Duration::from_secs_f64(v);
                println!("Deserializing {:?}", d);
                let res = SystemTime::UNIX_EPOCH + d;
                println!("Result {:?}", res);
                Ok(res)
            },
            Err(e) => Err(e),
        }
    }

    impl Event{
        pub fn new(code:u8, has_alt: bool, has_capslock: bool, has_ctrl: bool, has_meta: bool, has_shift: bool, time: SystemTime, press: bool, event_type: EventType, x: f64, y: f64, name: Option<String>) -> Result<Event, EventError>{

            if x < 0.0 || x > 1.0 || y < 0.0 || y > 1.0{
                return Err(EventError::InvalidCoordinates);
            }
            if let Some(ename) = &name{
                if ename == ""{
                    return Err(EventError::InvalidName);  // "Don't use empty string for events."
                }
                if ename != " " && ename.contains(" "){
                    return Err(EventError::InvalidName);  // "Don't use Spaces in complex event names"
                }
            }

            return Ok(Event{
                code,
                has_alt,
                has_capslock,
                has_ctrl,
                has_meta,
                has_shift,
                time,
                press,
                event_type,
                x,
                y,
                name,
            });
        }
    }

    pub fn format_events(events: Vec<Event>) -> String{
        let mut s = String::from("");
        for event in events{
            if !event.press{
                continue
            }
            if let Some(event_name) = event.name{
                s.push_str(&event_name);
            }

        }
        return s
    }
}




#[cfg(test)]
mod tests {
    use std::time::{SystemTime};
    use crate::rdev::{Event, EventError, EventType, format_events};

    fn simple_key(code: u8, name: Option<String>) -> Event{
        Event::new(
             code,
             false,
             false,
             false,
             false,
             false,
             SystemTime::now(),
             true,
             EventType::Key,
             0.,
             0.,
             name,
        ).unwrap()
    }

    #[test]
    fn format_simple() {
        let event = simple_key(1, Some(String::from("H")));
        let event2 = simple_key(2, Some(String::from("e")));
        assert_eq!(format_events(vec![event, event2]), String::from("He"));
    }

    #[test]
    fn event_out_of_boundary() {
        let event = Event::new(
            1,
            false,
            false,
            false,
            false,
            false,
            SystemTime::now(),
            true,
            EventType::Key,
            -1.,
            0.,
            Some(String::from("e")),
        );
        assert_eq!(event.err(), Some(EventError::InvalidCoordinates), "We should crash because x is negative");
    }

    #[test]
    fn event_invalid_name() {
        let event = Event::new(
            1,
            false,
            false,
            false,
            false,
            false,
            SystemTime::now(),
            true,
            EventType::Key,
            0.,
            0.,
            Some(String::from("")),
        );
        assert_eq!(event.err(), Some(EventError::InvalidName), "We should not emit empty string names");
    }

    #[test]
    fn event_invalid_name_with_space() {
        let event = Event::new(
            1,
            false,
            false,
            false,
            false,
            false,
            SystemTime::now(),
            true,
            EventType::Key,
            0.,
            0.,
            Some(String::from("Some event")),
        );
        assert_eq!(event.err(), Some(EventError::InvalidName), "We should not emit an event associated with a space");
    }

    #[test]
    fn event_json_deserialization() {
        let data = r#"
        {
            "keycode": 1,
            "hasAlt": false,
            "hasCapsLock": false,
            "hasCtrl": false,
            "hasMeta": false,
            "hasShift": false,
            "time": 15806656262720,
            "press": true,
            "event_type": "KEY",
            "x":0.0,
            "y":0.0
        }"#;

        // Parse the string of data into serde_json::Value.
        let event: Event = serde_json::from_str(data).unwrap();

        assert_eq!(event.code, 1);
    }

    #[test]
    fn event_json_serialization() {
        let event = Event::new(
            1,
            false,
            false,
            false,
            false,
            false,
            SystemTime::UNIX_EPOCH,
            true,
            EventType::Key,
            0.,
            0.,
            None,
        ).unwrap();
        let data = r#"{"keycode":1,"hasAlt":false,"hasCapsLock":false,"hasCtrl":false,"hasMeta":false,"hasShift":false,"time":0.0,"press":true,"event_type":"KEY","x":0.0,"y":0.0,"name":null}"#;

        // Parse the string of data into serde_json::Value.
        let output: String = serde_json::to_string(&event).unwrap();
        assert_eq!(output, data);
    }
}
