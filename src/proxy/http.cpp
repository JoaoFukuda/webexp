#include "http.hpp"

#include <iostream>
#include <algorithm>
#include <thread>
#include <chrono>

#include <memory.h>
#include <sys/socket.h>
#include <arpa/inet.h>
#include <unistd.h>
#include <netdb.h>

void handle_client(int client_stream);
int connect_to_server(std::string address, int port);

namespace proxy
{
	void HttpProxy::handle_next_connection() const {
		sockaddr_in client_hints;
		socklen_t client_len = sizeof(client_hints);
		int client;
		if ((client = accept(sock, reinterpret_cast<sockaddr*>(&client_hints), &client_len)) != -1) {
			std::thread(::handle_client, client).detach();
		}
	}

	HttpProxy::HttpProxy(int port) {
		sock = socket(AF_INET, SOCK_STREAM, 0);
		if (sock == -1) {
				throw("Could not create socket");
		}
		{
			int opt = 1;
			if (setsockopt(sock, SOL_SOCKET, SO_REUSEADDR, &opt, sizeof(opt)) == -1) {
				throw("Could not set socket options");
			}
		}

		sockaddr_in hint;
		hint.sin_addr.s_addr = INADDR_ANY;
		hint.sin_port = htons(port);
		hint.sin_family = AF_INET;

		if (bind(sock, reinterpret_cast<sockaddr*>(&hint), sizeof(hint)) == -1) {
			throw("Could not bind server");
		}

		if (listen(sock, 16) == -1) {
			throw("Could not put server to listen");
		}
	}
}

void connect_streams(int input, int output)
{
	uint8_t buf[BUFFER_SIZE];
	int datalen;

	memset(buf, 0, BUFFER_SIZE);
	while ((datalen = recv(input, buf, BUFFER_SIZE, 0))) {
		send(output, buf, datalen, 0);
		memset(buf, 0, BUFFER_SIZE);
	}
	send(output, buf, datalen, 0);
}

int connect_to_server(std::string address, int port)
{
	addrinfo *dns_results = nullptr;
	getaddrinfo(address.c_str(), nullptr, nullptr, &dns_results);

	if (dns_results == nullptr) {
		std::cout << "\e[1;91m[ERROR]\e[m DNS resolve did not return any known address" << std::endl;
		return -1;
	}

	sockaddr_in hint;
	hint.sin_addr.s_addr = reinterpret_cast<sockaddr_in*>(dns_results->ai_addr)->sin_addr.s_addr;
	hint.sin_port = htons(port);
	hint.sin_family = AF_INET;

	freeaddrinfo(dns_results);

	int server_stream = socket(AF_INET, SOCK_STREAM, 0);
	if (server_stream == -1) {
		std::cout << "\e[1;91m[ERROR]\e[m Could not create socket" << std::endl;
		return -1;
	}

	int connect_result = connect(server_stream, reinterpret_cast<sockaddr*>(&hint), sizeof(hint));

	if (connect_result == -1) {
		std::cout << "\e[1;91m[ERROR]\e[m Connection to " << address << ':' << port << " failed. errorno = " << errno << std::endl;
		return -1;
	}
	std::cout << "\e[1;92m[SUCCESS]\e[m Connection to " << address << ':' << port << " was established" << std::endl;
	return server_stream;
}

void handle_http_request(int client_stream, http::Request request_info)
{
	std::cout << "\e[1;94m[INFO]\e[m HTTP connection request to resource " << request_info.host << ':' << request_info.port << request_info.uri << std::endl;

	int server_stream = connect_to_server(request_info.host, request_info.port);
	if (server_stream) {
		std::string ok_message = "HTTP/1.1 500 ERROR\r\n\r\n";
		send(client_stream, ok_message.c_str(), ok_message.size(), 0);
		close(client_stream);
	}

	std::string request_text = request_info.create_request();

	std::cout << "Sending request:\n```\n" << request_text << "\n```" << std::endl;

	std::vector<uint8_t> buf (1024*4, 0);
	send(server_stream, request_text.c_str(), request_text.size(), 0);
	using namespace std::chrono_literals;
	std::this_thread::sleep_for(50ms);
	int buflen = recv(server_stream, buf.data(), buf.size(), 0);
	std::cout << "Got back response:\n```\n" << reinterpret_cast<char*>(buf.data()) << "\n```" << std::endl;
	send(client_stream, buf.data(), buflen, 0);
	close(client_stream);
}

void handle_tcp_connection(int client_stream, http::Request request_info)
{
	std::cout << "\e[1;94m[INFO]\e[m TCP connection request to server " << request_info.host << ':' << request_info.port << std::endl;

	int server_stream = connect_to_server(request_info.host, request_info.port);
	if (server_stream) {
		std::string ok_message = "HTTP/1.1 500 ERROR\r\n\r\n";
		send(client_stream, ok_message.c_str(), ok_message.size(), 0);
		close(client_stream);
	}

	std::string ok_message = "HTTP/1.1 200 OK\r\n\r\n";
	send(client_stream, ok_message.c_str(), ok_message.size(), 0);

	std::thread server_to_client([=] {
		connect_streams(server_stream, client_stream);
	});

	std::thread client_to_server([=] {
		connect_streams(client_stream, server_stream);
	});

	client_to_server.join();
	server_to_client.join();
}

void handle_client(int client_stream)
{
	std::vector<uint8_t> buf(BUFFER_SIZE, 0);

	if (recv(client_stream, buf.data(), buf.size(), 0)) {
		http::Request request_info(buf);
		if (request_info.method == "CONNECT")
			handle_tcp_connection(client_stream, request_info);
		else {
			handle_http_request(client_stream, request_info);
		}
		memset(buf.data(), 0, buf.size() * sizeof(buf.front()));
	}
}

