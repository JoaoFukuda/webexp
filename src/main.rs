use std::{ net, thread };

mod proxy;

fn main()
{
	let host = net::TcpListener::bind("0.0.0.0:3128").unwrap();
	println!("Starting proxy server on 0.0.0.0:3128!");

	for client in host.incoming()
	{
		thread::Builder::new().name("client handling".to_string()).spawn(move|| proxy::handle_client(client.unwrap())).unwrap();
	}
}
