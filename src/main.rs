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
	headers: Vec<(String, String)>,
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
			headers: vec![],
		};
	}

	pub fn create_from_proxy(request_text: String) -> Result<Request, ()>
	{
		let request_re = regex::Regex::new(r"^(\S*) ((https?)://)?(\S*?)(:(\d{1,5}))?(/\S*)? (\S*)").unwrap();
		let mut request = Request::create_empty();
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

	fn is_connection(&self) -> bool
	{
		return self.method.to_uppercase() == "CONNECT";
	}

	pub fn get_connection(&self) -> net::TcpStream
	{
		return net::TcpStream::connect(format!("{}:{}", self.host, self.port)).unwrap();
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

fn handle_tcp_connection(mut client: net::TcpStream, request: Request)
{
	if let Ok(mut server) = net::TcpStream::connect(format!("{}:{}", request.host, request.port))
	{
		client.write(String::from("HTTP/1.1 200 OK\r\n\r\n").as_bytes()).unwrap();

		let mut client_c = client.try_clone().unwrap();
		let mut server_c = server.try_clone().unwrap();

		let server_to_client = thread::spawn(move|| {
			let buf = &mut [0u8; 1024*1024];
			loop
			{
				match server_c.read(buf)
				{
					Ok(data_size) =>
					{
						
						if let Err(_) = client_c.write(&buf[0 .. data_size])
						{
							break;
						}
						if data_size == 0
						{
							break;
						}
					},
					_ =>
					{
						break;
					},
				}
			}
		});

		let client_to_server = thread::spawn(move|| {
			let buf = &mut [0u8; 1024*1024];
			loop
			{
				match client.read(buf)
				{
					Ok(data_size) =>
					{
						if let Err(_) = server.write(&buf[0 .. data_size])
						{
							break;
						}
						if data_size == 0 { break; }
					},
					_ =>
					{
						break;
					},
				}
			}
		});

		client_to_server.join().unwrap();
		server_to_client.join().unwrap();
	}
}

fn handle_client(mut client: net::TcpStream)
{
	let mut request: Request;
	let buf = &mut [0u8; 1024*1024];

	loop
	{
		if let Ok(data_size) = client.read(buf)
		{
			request = Request::create_from_proxy(String::from(str::from_utf8(&buf[0 .. data_size]).unwrap())).unwrap();

			if request.is_connection()
			{
				handle_tcp_connection(client, request);
				break;
			}
			else
			{
				let mut server_stream = request.get_connection();
				let request_text = format!("{}", request);

				server_stream.write(request_text.as_bytes()).unwrap();

				let mut response = String::new();
				loop {
					if let Ok(data_size) = server_stream.read(buf)
					{
						response = response + str::from_utf8(&buf[0 .. data_size]).unwrap();
					}

					if true
					{
						break;
					}
				}

				client.write(response.as_bytes()).unwrap();
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
			Connection: keep-alive\r\n
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
			Connection: keep-alive\r\n
			\r\n";

		let request = Request::create_from_proxy(request.to_string()).unwrap();
		assert_eq!(request.host, String::from("example.com"));
		assert_eq!(request.port, String::from("80"));
		assert_eq!(request.method, String::from("GET"));
		assert_eq!(request.uri, String::from("/"));
		assert_eq!(request.version, String::from("HTTP/1.1"));

		let request =
			"CONNECT https://example.com:443/cgi-bin/sumthing/run HTTP/2\r\n
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
			Connection: keep-alive\r\n
			\r\n";

		let request = Request::create_from_proxy(request.to_string()).unwrap();
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

		let request = Request::create_from_proxy(request.to_string()).unwrap();
		assert_eq!(format!("{}", request), String::from(
			"GET / HTTP/1.1\r\nHost: example.com\r\nConnection: keep-alive\r\n\r\n"
		));
	}
}

