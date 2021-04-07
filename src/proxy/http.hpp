#pragma once

#include "../proxy.hpp"
#include "../http/request.hpp"

#include <thread>

namespace proxy
{
	class HttpProxy : public Proxy
	{
		private:
			int sock;

		public:
			void handle_next_connection() const;

			HttpProxy() = default;
			HttpProxy(int);
	};
}

