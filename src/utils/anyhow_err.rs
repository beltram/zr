use std::fmt::{Debug, Display};

pub trait ErrConversion<T> {
    fn wrap(self) -> anyhow::Result<T>;
}

impl<T, E> ErrConversion<T> for Result<T, E>
    where E: Debug + Display + Send + Sync + 'static {
    fn wrap(self) -> anyhow::Result<T> {
        self.map_err(anyhow::Error::msg)
    }
}

pub trait OptConversion<T> {
    fn wrap<M>(self, msg: M) -> anyhow::Result<T>
        where M: AsRef<str> + Debug + Display + Send + Sync + 'static;
}

impl<T> OptConversion<T> for Option<T> {
    fn wrap<M>(self, msg: M) -> anyhow::Result<T>
        where M: AsRef<str> + Debug + Display + Send + Sync + 'static {
        self.ok_or_else(|| anyhow::Error::msg(msg))
    }
}