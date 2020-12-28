use std::{ net, str, thread, fmt };
use std::io::{ Read, Write };
use regex;

pub struct Request
{
	host: String,
	port: String,
	method: String,
	uri: String,
	version: String,
	_headers: Vec<(String, String)>,
}

impl Request
{
	pub fn create_empty() -> Request
	{
		return Request {
			host: String::new(),
			port: String::new(),
			method: String::new(),
			uri: String::new(),
			version: String::new(),
			_headers: vec![],
		};
	}

	pub fn create_from_proxy(request_text: String) -> Result<Request, ()>
	{
		let request_re = regex::Regex::new(r"^(\S*) ((https?)://)?(\S*?)(:(\d{1,5}))?(/\S*)? (\S*)").unwrap();
		let mut request = Request
		{
			host: String::new(),
			port: String::from("80"),
			method: String::new(),
			uri: String::from("/"),
			version: String::new(),
			_headers: vec![],
		};

		let var = request_re.captures(&request_text).unwrap();
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

		return Ok(request);
	}

	pub fn replay(&self) -> String
	{
		let mut end_host = net::TcpStream::connect(format!("{}:{}", self.host, self.port)).unwrap();
		let request_text = format!("{}", self);
		println!("Sending request:\n{}\n", request_text);
		end_host.write(request_text.as_bytes()).unwrap();

		let buf = &mut [0u8; 4096];
		end_host.read(buf).unwrap();
		println!("Receiving response:\n{}\n", str::from_utf8(buf).unwrap());

		return String::from(str::from_utf8(buf).unwrap());
	}

	fn is_connection(&self) -> bool
	{
		return self.method.to_uppercase() == "CONNECT";
	}
}

impl fmt::Display for Request
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		return write!(f, "{} {} {}\r\nHost: {}\r\nConnection: keep-alive\r\n\r\n", self.method, self.uri, self.version, self.host);
	}
}

fn handle_tcp_connection(mut client: net::TcpStream, request: Request) -> !
{
	if let Ok(mut server) = net::TcpStream::connect(format!("{}:{}", request.host, request.port))
	{
		client.write(String::from("HTTP/1.1 200 OK\r\n\r\n").as_bytes()).unwrap();

		client.set_read_timeout(Some(std::time::Duration::from_millis(200))).unwrap();
		server.set_read_timeout(Some(std::time::Duration::from_millis(200))).unwrap();

		let buf = &mut [0u8; 4096];
		loop
		{
			match server.read(buf)
			{
				Ok(data_size) if data_size != 0 => { client.write(buf).unwrap(); },
				Ok(_) => { break; },
				_ => {  },
			}

			match client.read(buf)
			{
				Ok(data_size) if data_size != 0 => { server.write(buf).unwrap(); },
				Ok(_) => { break; },
				_ => {  },
			}
		}
	}
	std::process::exit(0);
}

fn handle_client(mut client: net::TcpStream)
{
	let buf = &mut [0u8; 4096];
	let mut request: Request;

	loop
	{
		if let Ok(_) = client.read(buf)
		{
			request = Request::create_from_proxy(String::from(str::from_utf8(buf).unwrap())).unwrap();

			if request.is_connection()
			{
				handle_tcp_connection(client, request);
			}
			else
			{
				client.write(request.replay().as_bytes()).unwrap();
			}
		}
	}
}

fn main()
{
	let host = net::TcpListener::bind("0.0.0.0:3128").unwrap();
	println!("Starting proxy server on 0.0.0.0:3128!");

	for client in host.incoming()
	{
		thread::spawn(move|| handle_client(client.unwrap()));
	}
}

#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn test_request_connection_detection()
	{
		let mut request = Request::create_empty();
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
			\r\n";

		let request = Request::create_from_proxy(request.to_string()).unwrap();
		assert_eq!(request.host, String::from("example.com"));
		assert_eq!(request.port, String::from("80"));
		assert_eq!(request.method, String::from("GET"));
		assert_eq!(request.uri, String::from("/index.php"));
		assert_eq!(request.version, String::from("HTTP/1.1"));

		let request =
			"GET http://example.com/ HTTP/1.1\r\n
			Host: example.com\r\n
			\r\n";

		let request = Request::create_from_proxy(request.to_string()).unwrap();
		assert_eq!(request.host, String::from("example.com"));
		assert_eq!(request.port, String::from("80"));
		assert_eq!(request.method, String::from("GET"));
		assert_eq!(request.uri, String::from("/"));
		assert_eq!(request.version, String::from("HTTP/1.1"));

		let request =
			"CONNECT https://example.com:443/cgi-bin/sumthing/run HTTP/2\r\n
			Host: example.com\r\n
			\r\n";

		let request = Request::create_from_proxy(request.to_string()).unwrap();
		assert_eq!(request.host, String::from("example.com"));
		assert_eq!(request.port, String::from("443"));
		assert_eq!(request.method, String::from("CONNECT"));
		assert_eq!(request.uri, String::from("/cgi-bin/sumthing/run"));
		assert_eq!(request.version, String::from("HTTP/2"));

		let request =
			"GET example.com/ HTTP/1.1\r\n
			Host: example.com\r\n
			\r\n";

		let request = Request::create_from_proxy(request.to_string()).unwrap();
		assert_eq!(request.host, String::from("example.com"));
		assert_eq!(request.port, String::from("80"));
		assert_eq!(request.method, String::from("GET"));
		assert_eq!(request.uri, String::from("/"));
		assert_eq!(request.version, String::from("HTTP/1.1"));
	}
}

