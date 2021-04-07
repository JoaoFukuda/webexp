#pragma once

#include <cstdint>
#include <vector>

constexpr int BUFFER_SIZE = 1024 * 4; // 4KB

namespace proxy
{
	class Proxy {
		public:
			virtual void handle_next_connection() const = 0;
	};
}

