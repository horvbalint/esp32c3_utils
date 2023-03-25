use std::fmt::Debug;

pub trait LogError<O, E: Debug> {
    fn log(self) -> Result<O, E>;
}

impl<O, E: Debug> LogError<O, E> for Result<O, E> {
    fn log(self) -> Result<O, E> {
        match &self {
            Ok(_) => self,
            Err(err) => {
                dbg!(&err);
                self
            }
        }
    }
}