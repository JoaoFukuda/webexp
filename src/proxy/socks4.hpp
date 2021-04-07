#pragma once

#include "../proxy.hpp"
#include "../http/request.hpp"

namespace proxy
{
	struct Socks4Request {
		uint8_t version;
		uint8_t command;
		uint16_t dest_port;
		uint32_t dest_ip;
	};

	struct Socks4Response {
		uint8_t version;
		uint8_t reply;
		uint8_t ignore[6];
	};

	class Socks4Proxy : public Proxy
	{
		private:
			int sock;

			void handle_client(int sock) const;
			void connect_sockets(int in, int out) const;

		public:
			void handle_next_connection() const;

			Socks4Proxy() = default;
			Socks4Proxy(int);
	};
}

