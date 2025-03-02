use std::sync::atomic::{AtomicU32, Ordering};

use serde::{Serialize, Serializer};

/// Atomic counter used to generate unique message IDs.
static MESSAGE_ID: AtomicU32 = AtomicU32::new(0);

/// Represents the direction of a Marionette message.
///
/// - `Request`: Indicates that the message is a request.
/// - `Response`: Indicates that the message is a response.
#[derive(Debug)]
pub enum Direction {
    Request = 0,
    Response = 1,
}

/// Implements custom serialization for [`Direction`] by serializing its numeric value.
impl Serialize for Direction {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let value = match self {
            Self::Request => 0,
            Self::Response => 1,
        };

        value.serialize(serializer)
    }
}

type Id = u32;
type Name = String;

/// Represents a Marionette command containing a direction, a unique ID, a command name, and associated data.
///
/// The [`Command`] struct is used to encapsulate a command message sent to or received from Marionette.
#[derive(Debug, Serialize)]
pub struct Command<T>(Direction, Id, Name, T);

impl<T> Command<T> {
    pub fn new<C>(direction: Direction, command: C, data: T) -> Self
    where
        C: Into<String>,
    {
        Self(
            direction,
            MESSAGE_ID.fetch_add(1, Ordering::SeqCst),
            command.into(),
            data,
        )
    }

    pub fn new_request<C>(command: C, data: T) -> Self
    where
        C: Into<String>,
    {
        Self::new(Direction::Request, command.into(), data)
    }

    #[must_use]
    pub const fn id(&self) -> Id {
        self.1
    }
}

#[cfg(test)]
#[cfg_attr(coverage, coverage(off))]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_serialization() {
        let request = serde_json::to_string(&Direction::Request).unwrap();
        let response = serde_json::to_string(&Direction::Response).unwrap();

        assert_eq!(request, "0", "Direction::Request should serialize to 0");
        assert_eq!(response, "1", "Direction::Response should serialize to 1");
    }

    #[test]
    fn test_command_id_increment() {
        let command_1 = Command::new(Direction::Request, "request-test-1", 101);
        let command_2 = Command::new(Direction::Response, "response-test-2", 102);
        let command_3 = Command::new(Direction::Request, "request-test-3", 103);

        assert!(
            command_1.id() < command_2.id(),
            "Command IDs should increment"
        );
        assert!(
            command_2.id() < command_3.id(),
            "Command IDs should increment"
        );
    }

    #[test]
    fn test_request_serialization() {
        let command = Command::new_request("request-serialize", ("hello", "world"));
        let serialized = serde_json::to_string(&command).unwrap();

        let expected = format!(
            "[0,{},\"request-serialize\",[\"hello\",\"world\"]]",
            command.id()
        );
        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_response_serialization() {
        #[derive(Serialize)]
        struct Response {
            message: String,
            count: u8,
        }

        let command = Command::new(
            Direction::Response,
            "response-serialize",
            Response {
                message: "hello world".into(),
                count: 42,
            },
        );
        let serialized = serde_json::to_string(&command).unwrap();

        let expected = format!(
            "[1,{},\"response-serialize\",{{\"message\":\"hello world\",\"count\":42}}]",
            command.id()
        );
        assert_eq!(serialized, expected);
    }
}
