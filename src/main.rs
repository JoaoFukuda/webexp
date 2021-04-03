use std::{ net, thread };

use proxy::http::HttpProxy;

mod proxy;
mod http;
mod tls;

fn main()
{
	let proxy: HttpProxy = HttpProxy::new("0.0.0.0:3128");

	for client in proxy.incomming() {
		thread::spawn(move|| proxy::handle_client(client.unwrap()));
	}
}

