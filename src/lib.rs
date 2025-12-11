pub mod codec;
pub mod error;
pub mod esl;
pub mod event;

#[cfg(test)]
mod tests {
    #[test]
    fn run() {
        assert_eq!(super::codec::find_header_end(b"\n\n"), Some(0));
        assert_eq!(super::codec::find_header_end(b"\r\n\r\n"), Some(0));
    }
}
