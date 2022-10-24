mod proto;
mod device;
mod client;
mod error;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use crate::client::AdbClient;
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
    #[test]
    fn test_connect() {
        let adb = AdbClient::new(String::from("localhost"), 5037, 10);
        let client = adb.connect();
        println!("{:?}", client)
    }
}
