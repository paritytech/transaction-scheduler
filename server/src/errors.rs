use jsonrpc_core::{Error, ErrorCode, Value};

pub fn rlp<T: ::std::fmt::Display>(error: T) -> Error {
	Error {
		code: ErrorCode::InvalidParams,
		message: "Invalid RLP.".into(),
		data: Some(Value::String(format!("{}", error))),
	}
}
pub fn transaction<T: ::std::fmt::Display>(error: T) -> Error {
	Error {
		code: ErrorCode::InvalidParams,
		message: "Invalid Transaction.".into(),
		data: Some(Value::String(format!("{}", error))),
	}
}
pub fn block<T: ::std::fmt::Display>(error: T) -> Error {
	Error {
		code: ErrorCode::InvalidParams,
		message: "Invalid block number.".into(),
		data: Some(Value::String(format!("{}", error))),
	}
}
pub fn timestamp<T: ::std::fmt::Display>(error: T) -> Error {
	Error {
		code: ErrorCode::InvalidParams,
		message: "Invalid timestamp.".into(),
		data: Some(Value::String(format!("{}", error))),
	}
}
pub fn internal<T: ::std::fmt::Display>(error: T) -> Error {
	Error {
		code: ErrorCode::InternalError,
		message: "Internal Error".into(),
		data: Some(Value::String(format!("{}", error))),
	}
}
