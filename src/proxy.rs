mod http;
mod socks4;

trait Proxy {
	fn recv<'a>() -> &'a [u8];
	fn send(data: &[u8]);
}

