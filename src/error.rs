//
// Error generator macro
//
use std::error::Error as StdError;


macro_rules! make_error {
    ( $( $name:ident ( $ty:ty ): $src_fn:expr, $usr_msg_fun:expr ),+ $(,)? ) => {
        const BAD_REQUEST: u16 = 400;

        pub enum ErrorKind { $($name( $ty )),+ }

        #[derive(Debug)]
        pub struct ErrorEvent { pub event: EventType }
        pub struct Error { message: String, error: ErrorKind, error_code: u16, event: Option<ErrorEvent> }

        $(impl From<$ty> for Error {
            fn from(err: $ty) -> Self { Error::from((stringify!($name), err)) }
        })+
        $(impl<S: Into<String>> From<(S, $ty)> for Error {
            fn from(val: (S, $ty)) -> Self {
                Error { message: val.0.into(), error: ErrorKind::$name(val.1), error_code: BAD_REQUEST, event: None }
            }
        })+
        impl StdError for Error {
            fn source(&self) -> Option<&(dyn StdError + 'static)> {
                match &self.error {$( ErrorKind::$name(e) => $src_fn(e), )+}
            }
        }
        impl std::fmt::Display for Error {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match &self.error {$(
                   ErrorKind::$name(e) => f.write_str(&$usr_msg_fun(e, &self.message)),
                )+}
            }
        }
    };
}

use diesel::r2d2::PoolError as R2d2Err;

#[derive(Serialize)]
pub struct Empty {}

// Error struct
// Contains a String error message, meant for the user and an enum variant, with an error of different types.
//
// After the variant itself, there are two expressions. The first one indicates whether the error contains a source error (that we pretty print).
// The second one contains the function used to obtain the response sent to the client
make_error! {
    // Just an empty error
    Empty(Empty):     _no_source, _serialize,
    // Used to represent err! calls
    Simple(String):  _no_source,  _api_error,
    // Used for special return values, like 2FA errors

    DieselCon(DieselConErr): _has_source, _api_error,
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.source() {
            Some(e) => write!(f, "{}.\n[CAUSE] {:#?}", self.message, e),
            None => match self.error {
                ErrorKind::Empty(_) => Ok(()),
                ErrorKind::Simple(ref s) => {
                    if &self.message == s {
                        write!(f, "{}", self.message)
                    } else {
                        write!(f, "{}. {}", self.message, s)
                    }
                }
                _ => unreachable!(),
            },
        }
    }
}

impl Error {
    pub fn new<M: Into<String>, N: Into<String>>(usr_msg: M, log_msg: N) -> Self {
        (usr_msg, log_msg.into()).into()
    }

    pub fn empty() -> Self {
        Empty {}.into()
    }

    #[must_use]
    pub fn with_msg<M: Into<String>>(mut self, msg: M) -> Self {
        self.message = msg.into();
        self
    }

    #[must_use]
    pub const fn with_code(mut self, code: u16) -> Self {
        self.error_code = code;
        self
    }

    #[must_use]
    pub fn with_event(mut self, event: ErrorEvent) -> Self {
        self.event = Some(event);
        self
    }

    pub fn get_event(&self) -> &Option<ErrorEvent> {
        &self.event
    }
}

pub trait MapResult<S> {
    fn map_res(self, msg: &str) -> Result<S, Error>;
}

impl<S, E: Into<Error>> MapResult<S> for Result<S, E> {
    fn map_res(self, msg: &str) -> Result<S, Error> {
        self.map_err(|e| e.into().with_msg(msg))
    }
}

impl<E: Into<Error>> MapResult<()> for Result<usize, E> {
    fn map_res(self, msg: &str) -> Result<(), Error> {
        self.and(Ok(())).map_res(msg)
    }
}

impl<S> MapResult<S> for Option<S> {
    fn map_res(self, msg: &str) -> Result<S, Error> {
        self.ok_or_else(|| Error::new(msg, ""))
    }
}

const fn _has_source<T>(e: T) -> Option<T> {
    Some(e)
}
fn _no_source<T, S>(_: T) -> Option<S> {
    None
}

fn _serialize(e: &impl serde::Serialize, _msg: &str) -> String {
    serde_json::to_string(e).unwrap()
}

fn _api_error(_: &impl std::any::Any, msg: &str) -> String {
    let json = json!({
        "Message": msg,
        "error": "",
        "error_description": "",
        "ValidationErrors": {"": [ msg ]},
        "ErrorModel": {
            "Message": msg,
            "Object": "error"
        },
        "ExceptionMessage": null,
        "ExceptionStackTrace": null,
        "InnerExceptionMessage": null,
        "Object": "error"
    });
    _serialize(&json, "")
}

//
// Rocket responder impl
//
use std::io::Cursor;
use serde_json::json;



//
// Error return macros
//
#[macro_export]
macro_rules! err {
    ($msg:expr) => {{
        error!("{}", $msg);
        return Err($crate::error::Error::new($msg, $msg));
    }};
    ($msg:expr, ErrorEvent $err_event:tt) => {{
        error!("{}", $msg);
        return Err($crate::error::Error::new($msg, $msg).with_event($crate::error::ErrorEvent $err_event));
    }};
    ($usr_msg:expr, $log_value:expr) => {{
        error!("{}. {}", $usr_msg, $log_value);
        return Err($crate::error::Error::new($usr_msg, $log_value));
    }};
    ($usr_msg:expr, $log_value:expr, ErrorEvent $err_event:tt) => {{
        error!("{}. {}", $usr_msg, $log_value);
        return Err($crate::error::Error::new($usr_msg, $log_value).with_event($crate::error::ErrorEvent $err_event));
    }};
}

#[macro_export]
macro_rules! err_silent {
    ($msg:expr) => {{
        return Err($crate::error::Error::new($msg, $msg));
    }};
    ($usr_msg:expr, $log_value:expr) => {{
        return Err($crate::error::Error::new($usr_msg, $log_value));
    }};
}

#[macro_export]
macro_rules! err_code {
    ($msg:expr, $err_code:expr) => {{
        error!("{}", $msg);
        return Err($crate::error::Error::new($msg, $msg).with_code($err_code));
    }};
    ($usr_msg:expr, $log_value:expr, $err_code:expr) => {{
        error!("{}. {}", $usr_msg, $log_value);
        return Err($crate::error::Error::new($usr_msg, $log_value).with_code($err_code));
    }};
}

#[macro_export]
macro_rules! err_discard {
    ($msg:expr, $data:expr) => {{
        std::io::copy(&mut $data.open(), &mut std::io::sink()).ok();
        return Err($crate::error::Error::new($msg, $msg));
    }};
    ($usr_msg:expr, $log_value:expr, $data:expr) => {{
        std::io::copy(&mut $data.open(), &mut std::io::sink()).ok();
        return Err($crate::error::Error::new($usr_msg, $log_value));
    }};
}

#[macro_export]
macro_rules! err_json {
    ($expr:expr, $log_value:expr) => {{
        return Err(($log_value, $expr).into());
    }};
    ($expr:expr, $log_value:expr, $err_event:expr, ErrorEvent) => {{
        return Err(($log_value, $expr).into().with_event($err_event));
    }};
}

#[macro_export]
macro_rules! err_handler {
    ($expr:expr) => {{
        error!(target: "auth", "Unauthorized Error: {}", $expr);
        return ::rocket::request::Outcome::Failure((rocket::http::Status::Unauthorized, $expr));
    }};
    ($usr_msg:expr, $log_value:expr) => {{
        error!(target: "auth", "Unauthorized Error: {}. {}", $usr_msg, $log_value);
        return ::rocket::request::Outcome::Failure((rocket::http::Status::Unauthorized, $usr_msg));
    }};
}