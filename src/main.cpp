#include "proxy/socks4.hpp"

#include <thread>
#include <iostream>

int main()
{
	int port = 3128;
	proxy::Socks4Proxy proxy(port);
	std::cout << "\033[1;94m[INFO]\033[m Proxy successfully bound to port " << port << std::endl;

	std::cout << "\033[1;94m[INFO]\033[m Awaiting connection requests" << std::endl;
	while (true) {
		proxy.handle_next_connection();
	}
}

