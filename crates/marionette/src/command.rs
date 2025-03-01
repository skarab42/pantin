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
