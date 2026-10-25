use crate::field::{FieldErrors, Fields};

pub trait DeserializeFormUrlEncoded: Sized {
    fn deserialize(fields: Fields) -> Result<Self, FieldErrors>;
}
