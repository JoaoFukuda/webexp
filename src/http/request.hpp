#pragma once

#include <string>
#include <map>
#include <vector>

namespace http
{
	class Request {
		public:
			int port;
			std::string host;
			std::string uri;
			std::string method;
			std::string protocol;
			std::string version;
			std::map<std::string, std::string> headers;
			std::map<std::string, std::string> cookies;

			std::string create_request();

			Request(std::vector<uint8_t>);
	};
}

