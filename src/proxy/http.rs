use std::{ net, fmt };
use regex;

pub struct Request
{
	pub host: String,
	pub port: String,
	pub method: String,
	pub uri: String,
	pub version: String,
	pub headers: Vec<(String, String)>,
	pub _body: String,
}

impl Request
{
	pub fn new() -> Request
	{
		return Request {
			host: String::new(),
			port: String::new(),
			method: String::new(),
			uri: String::new(),
			version: String::new(),
			headers: vec![],
			_body: String::new(),
		};
	}

	pub fn from_proxy(request_text: String) -> Result<Request, ()>
	{
		let mut request = Request::new();
		let request_re = regex::Regex::new(r"^(\S*) ((https?)://)?(\S*?)(:(\d{1,5}))?(/\S*)? (\S*)").unwrap();
		request.port = String::from("80");
		request.uri = String::from("/");

		if let Some(var) = request_re.captures(&request_text)
		{
			request.host = var.get(4).unwrap().as_str().to_string();
			request.version = var.get(8).unwrap().as_str().to_string();
			request.method = var.get(1).unwrap().as_str().to_string();
			if let Some(uri) = var.get(7)
			{
				request.uri = uri.as_str().to_string();
			}
			if let Some(port) = var.get(6)
			{
				request.port = port.as_str().to_string();
			}
		}
		else
		{
			println!("[ERR] Could not parse {:?}", request_text);
			return Err(());
		}

		let header_re = regex::Regex::new(r"^\s*(\S+):\s*(.*)$").unwrap();
		let mut is_header = true;
		for line in (&request_text).split("\r\n")
		{
			if is_header
			{
				is_header = false;
				continue;
			}
			if let Some(capture) = header_re.captures(line)
			{
				request.headers.push(
					(
						capture.get(1).unwrap().as_str().to_string(),
						capture.get(2).unwrap().as_str().to_string()
					));
			}
			else
			{
				break;
			}
		}

		return Ok(request);
	}

	pub fn is_connection(&self) -> bool
	{
		return self.method.to_uppercase() == "CONNECT";
	}

	pub fn get_connection(&self) -> Result<net::TcpStream, std::io::Error>
	{
		return net::TcpStream::connect(format!("{}:{}", self.host, self.port));
	}

	pub fn to_string(&self) -> String
	{
		let mut request_string = format!("{} {} {}\r\n", self.method, self.uri, self.version);

		for (title, value) in &self.headers
		{
			request_string = format!("{}{}: {}\r\n", request_string, title, value);
		}
		return format!("{}{}", request_string, "\r\n");
	}
}

impl fmt::Display for Request
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		write!(f, "{} {} {}\r\n", self.method, self.uri, self.version)?;
		for (title, value) in &self.headers
		{
			write!(f, "{}: {}\r\n", title, value)?;
		}
		return write!(f, "\r\n");
	}
}

#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn test_request_connection_detection()
	{
		let mut request = Request::new();
		assert_eq!(request.is_connection(), false);

		request.method = String::from("GET");
		assert_eq!(request.is_connection(), false);

		request.method = String::from("CONNECT");
		assert_eq!(request.is_connection(), true);

		request.method = String::from("connect");
		assert_eq!(request.is_connection(), true);

		request.method = String::from("CoNnEcT");
		assert_eq!(request.is_connection(), true);
	}

	#[test]
	fn test_request_from_proxy()
	{
		let request =
			"GET https://example.com:80/index.php HTTP/1.1\r\n
			Host: example.com\r\n
			Connection: keep-alive\r\n
			\r\n";

		let request = Request::from_proxy(request.to_string()).unwrap();
		assert_eq!(request.host, String::from("example.com"));
		assert_eq!(request.port, String::from("80"));
		assert_eq!(request.method, String::from("GET"));
		assert_eq!(request.uri, String::from("/index.php"));
		assert_eq!(request.version, String::from("HTTP/1.1"));

		let request =
			"GET http://example.com/ HTTP/1.1\r\n
			Host: example.com\r\n
			Connection: keep-alive\r\n
			\r\n";

		let request = Request::from_proxy(request.to_string()).unwrap();
		assert_eq!(request.host, String::from("example.com"));
		assert_eq!(request.port, String::from("80"));
		assert_eq!(request.method, String::from("GET"));
		assert_eq!(request.uri, String::from("/"));
		assert_eq!(request.version, String::from("HTTP/1.1"));

		let request =
			"CONNECT https://example.com:443/cgi-bin/sumthing/run HTTP/2\r\n
			\r\n";

		let request = Request::from_proxy(request.to_string()).unwrap();
		assert_eq!(request.host, String::from("example.com"));
		assert_eq!(request.port, String::from("443"));
		assert_eq!(request.method, String::from("CONNECT"));
		assert_eq!(request.uri, String::from("/cgi-bin/sumthing/run"));
		assert_eq!(request.version, String::from("HTTP/2"));

		let request =
			"GET example.com/ HTTP/1.1\r\n
			Host: example.com\r\n
			Connection: keep-alive\r\n
			\r\n";

		let request = Request::from_proxy(request.to_string()).unwrap();
		assert_eq!(request.host, String::from("example.com"));
		assert_eq!(request.port, String::from("80"));
		assert_eq!(request.method, String::from("GET"));
		assert_eq!(request.uri, String::from("/"));
		assert_eq!(request.version, String::from("HTTP/1.1"));
	}

	#[test]
	fn test_request_to_text()
	{
		let request =
			"GET example.com/ HTTP/1.1\r\n
			Host: example.com\r\n
			Connection: keep-alive\r\n
			\r\n";

		let request = Request::from_proxy(request.to_string()).unwrap();
		assert_eq!(format!("{}", request), String::from(
			"GET / HTTP/1.1\r\nHost: example.com\r\nConnection: keep-alive\r\n\r\n"
		));
	}
}
