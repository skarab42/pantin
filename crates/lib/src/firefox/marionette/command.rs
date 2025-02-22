use std::sync::atomic::{AtomicU32, Ordering};

use serde::{Serialize, Serializer};

static MESSAGE_ID: AtomicU32 = AtomicU32::new(0);

#[derive(Debug)]
pub enum Direction {
    Request = 0,
    Response = 1,
}

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
