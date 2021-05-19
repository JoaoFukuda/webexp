#pragma once

#include "../proxy.hpp"

namespace webexp
{
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
			int sock;

			public:
				Socks4Proxy() = default;
				Socks4Proxy(int);

				void handle_next_connection() const noexcept;

			private:
				void handle_client(int sock) const;
				void connect_sockets(int in, int out) const;
		};
	}
}

