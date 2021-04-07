#include "request.hpp"

#include <regex>
#include <sstream>

namespace http
{
	Request::Request(std::vector<uint8_t> raw_request) {
		std::regex request_re(R"(^(\S+) (?:(https?|tcp|ftp|ssh)://)?([\-\.a-zA-Z0-9]+)(?::(\d{1,5}))?((?:/[\-\.a-zA-Z0-9\?\&=]*)*)? (\S+))");
		std::regex headers_re(R"(^(.*?): ?(.*)\s)");

		std::stringstream request_stream(reinterpret_cast<char*>(raw_request.data()));

		std::string line;
		std::smatch regex_result;
		std::getline(request_stream, line);
		if (std::regex_search(line, regex_result, request_re)) {
			if (regex_result[1].matched) {
				method = regex_result[1];
			}
			if (regex_result[2].matched) {
				protocol = regex_result[2];
			}
			if (regex_result[3].matched) {
				host = regex_result[3];
			}
			if (regex_result[4].matched) {
				std::stringstream(regex_result[4]) >> port;
			}
			else {
				if (protocol == "https") {
					port = 443;
				}
				else if (protocol == "ftp") {
					port = 21;
				}
				else if (protocol == "ssh") {
					port = 22;
				}
				else
					port = 80;
			}
			if (regex_result[5].matched) {
				uri = regex_result[5];
			}
			if (regex_result[6].matched) {
				version = regex_result[6];
			}
		}

		while (std::getline(request_stream, line))
		{
			if (std::regex_search(line, regex_result, headers_re)) {
				headers.emplace(regex_result[1], regex_result[2]);
			}
		}
	}
	std::string Request::create_request() {
		std::stringstream request;

		request << method << ' ' << uri << ' ' << version << "\r\n";
		for (const auto & header : headers) {
			request << header.first << ": " << header.second << "\r\n";
		}

		request << "\r\n";

		return request.str();
	}
}

