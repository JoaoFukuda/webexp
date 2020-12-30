use std::{ net, str, thread };
use std::io::{ Read, Write };

mod http;

const BUFFER_SIZE: usize = 1024*1024;

fn connect_streams(mut input: net::TcpStream, mut output: net::TcpStream)
{
	let buf = &mut [0u8; BUFFER_SIZE];
	loop
	{
		match input.read(buf)
		{
			Ok(data_size) =>
			{
				if let Err(_) = output.write(&buf[0 .. data_size])
				{
					break;
				}
				if data_size == 0
				{
					return;
				}
			},
			_ =>
			{
				return;
			},
		}
	}
}

fn handle_http_request(mut client_stream: &net::TcpStream, request: http::Request)
{
	println!("[INF] HTTP: Sending {} to {}:{}", request.method, request.host, request.port);

	let buf = &mut [0u8; BUFFER_SIZE];
	let mut server_stream = request.get_connection().unwrap();

	server_stream.write(request.to_string().as_bytes()).unwrap();

	let mut response = String::new();
	loop {
		if let Ok(data_size) = server_stream.read(buf)
		{
			response = response + str::from_utf8(&buf[0 .. data_size]).unwrap();

			if data_size == 0 || response.ends_with("\x00\x00")
			{
				println!("broken");
				break;
			}
		}
		else
		{
			break;
		}
	}

	client_stream.write(response.as_bytes()).unwrap();
}

fn handle_tcp_connection(mut client_stream: net::TcpStream, request: http::Request)
{
	if let Ok(server_stream) = net::TcpStream::connect(format!("{}:{}", request.host, request.port))
	{
		client_stream.write(String::from("HTTP/1.1 200 OK\r\n\r\n").as_bytes()).unwrap();

		let client_stream_c = client_stream.try_clone().unwrap();
		let server_stream_c = server_stream.try_clone().unwrap();

		let server_to_client = thread::spawn(move|| {
			connect_streams(server_stream_c, client_stream_c);
		});

		let client_to_server = thread::spawn(move|| {
			connect_streams(client_stream, server_stream);
		});

		client_to_server.join().unwrap();
		server_to_client.join().unwrap();
	}
	else
	{
		client_stream.write(String::from("HTTP/1.1 500 ERROR\r\n\r\n").as_bytes()).unwrap();
	}
}

pub fn handle_client(mut client_stream: net::TcpStream)
{
	let request: http::Request;
	let buf = &mut [0u8; BUFFER_SIZE];

	if let Ok(data_size) = client_stream.read(buf)
	{
		if data_size != 0
		{
			let request_string = str::from_utf8(&buf[0 .. data_size]).unwrap();
			request = http::Request::from_proxy(request_string.to_string()).unwrap();

			if request.is_connection()
			{
				handle_tcp_connection(client_stream, request);
			}
			else
			{
				handle_http_request(&client_stream, request);
			}
		}
	}
}

