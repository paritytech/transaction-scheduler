use jsonrpc_core::{Error, ErrorCode, Value};

pub fn rlp<T: ::std::fmt::Debug>(error: T) -> Error {
	Error {
		code: ErrorCode::InvalidParams,
		message: "Invalid RLP.".into(),
		data: Some(Value::String(format!("{:?}", error))),
	}
}
pub fn transaction<T: ::std::fmt::Debug>(error: T) -> Error {
	Error {
		code: ErrorCode::InvalidParams,
		message: "Invalid Transaction.".into(),
		data: Some(Value::String(format!("{:?}", error))),
	}
}
