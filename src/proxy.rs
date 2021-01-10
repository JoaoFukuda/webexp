use std::{ net, str, thread };
use std::io::{ Read, Write };

mod request;

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

fn handle_http_request(mut _client_stream: &net::TcpStream, _raw_request: &[u8])
{
	println!("[INF] HTTP: Sending request");
}

fn handle_tcp_connection(mut client_stream: net::TcpStream, raw_request: &[u8])
{
	let request_info = request::Info::new(raw_request);
	if let Ok(server_stream) = net::TcpStream::connect(format!("{}:{}", request_info.host.unwrap(), request_info.port.unwrap()))
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
		return;
	}
	client_stream.write("HTTP/1.1 500 ERROR\r\n\r\n".as_bytes()).unwrap();
}

pub fn handle_client(mut client_stream: net::TcpStream)
{
	let buf = &mut [0u8; BUFFER_SIZE];

	if let Ok(data_size) = client_stream.read(buf)
	{
		if data_size != 0
		{
			if str::from_utf8(&buf[0 .. 7]).unwrap() == "CONNECT"
			{
				handle_tcp_connection(client_stream, buf);
			}
			else
			{
				handle_http_request(&client_stream, buf);
			}
		}
	}
}

