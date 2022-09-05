
macro_rules! unwrap_or_return_error {
    ( $result:expr, $error:expr ) => {
        match $result {
            Ok(value) => value,
            Err(e) => { println!("{:?}", e); return Err($error)},
        }
        
    }
}
pub(crate) use unwrap_or_return_error;



#[derive(Debug)]
pub enum Error{
    InternalError(String),
    SignatureDecryptionError(String),
    HTTPError(String),
    ParsingError(String)
}