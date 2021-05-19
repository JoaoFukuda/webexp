#pragma once

#include <cstdint>
#include <vector>

namespace webexp
{
	namespace proxy
	{
		constexpr int BUFFER_SIZE = 1024 * 4;
		class Proxy {
			public:
				virtual void handle_next_connection() const = 0;
		};
	}
}

