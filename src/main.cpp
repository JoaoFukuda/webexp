#include "proxy/socks4.hpp"

#include <thread>
#include <iostream>

using namespace webexp;

int main()
{
	int port = 3128;
	proxy::Socks4Proxy proxy(port);
	std::cout << "\e[1;94m[INFO]\e[m Proxy successfully bound to port " << port << std::endl;

	std::cout << "\e[1;94m[INFO]\e[m Awaiting connection requests" << std::endl;
	while (true) {
		proxy.handle_next_connection();
	}
}

