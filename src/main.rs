use std::{ net, str, thread, fmt };
use std::io::{ Read };
use regex;

fn handle_client(mut client: net::TcpStream)
{
	let buf = &mut [0u8; 4096];
	client.read(buf).unwrap();
	let request = Request::new(to_string(buf)).unwrap();
	println!("{}", request);
}

fn to_string(buf: &mut [u8]) -> String
{
	return str::from_utf8(buf).unwrap().to_string();
}

pub struct Request {
	method: String,
	url: String,
	version: String,
	_headers: Vec<(String, String)>,
}

impl Request {
	pub fn new(request_text: String) -> Result<Request, ()>
	{
		let request_re = regex::Regex::new(r"^(\S*) (\S*) (\S*)").unwrap();
		let mut request = Request {
			method: String::new(),
			url: String::new(),
			version: String::new(),
			_headers: vec![],
		};

		for var in request_re.captures_iter(&request_text) {
			request.method = (&var[1]).to_string();
			request.url = (&var[2]).to_string();
			request.version = (&var[3]).to_string();
		}

		return Ok(request);
	}
}

impl fmt::Display for Request {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		return write!(f, "M: {}\nU: {}\nV: {}", self.method, self.url, self.version);
	}
}

fn main()
{
	let host = net::TcpListener::bind("0.0.0.0:3128").unwrap();
	println!("Starting proxy server!");

	for client in host.incoming() {
		thread::spawn(move|| handle_client(client.unwrap()));
	}
}

