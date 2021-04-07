#include "socks4.hpp"

#include <iostream>
#include <thread>

#include <sys/socket.h>
#include <arpa/inet.h>
#include <unistd.h>
#include <memory.h>

constexpr int BUFSIZE = 1024;

namespace proxy
{
	enum Socks4ResponseCode {
		Granted = 0x5a,
		Rejected = 0x5b,
		ClientNotRunningIdentd = 0x5c,
		ClientIdentdCouldNotConfirmUID = 0x5d,
	};

	void Socks4Proxy::handle_client(int sock) const {
		Socks4Request request;
		recv(sock, &request, 8, 0);
		in_addr addr;
		addr.s_addr = request.dest_ip;
		std::cout << "\033[1;94m[INFO]\033[m Got request for"
			<< "\n\033[1;92m>\033[m Version: " << static_cast<int>(request.version)
			<< "\n\033[1;92m>\033[m Command: " << static_cast<int>(request.command)
			<< "\n\033[1;92m>\033[m Destination port: " << ntohs(request.dest_port)
			<< "\n\033[1;92m>\033[m Destination ip: " << inet_ntoa(addr) << std::endl;
		char buf[BUFSIZE];
		memset(buf, 0, BUFSIZE);
		recv(sock, buf, BUFSIZE, 0);
		std::cout << "\033[1;92m>\033[m ID: " << buf << std::endl;

		int server_sock = socket(AF_INET, SOCK_STREAM, 0);

		sockaddr_in server_hint;
		server_hint.sin_addr.s_addr = request.dest_ip;
		server_hint.sin_port = request.dest_port;
		server_hint.sin_family = AF_INET;

		Socks4Response response;

		if (connect(server_sock, reinterpret_cast<sockaddr*>(&server_hint), sizeof(server_hint)) == -1) {
			std::cerr << "\033[1;91m[ERROR]\033[m Could not connect to server at " << inet_ntoa(addr) << ':' << ntohs(request.dest_port) << std::endl;
			response.reply = Rejected;
			send(sock, &response, 8, 0);
			close(sock);
			return;
		}

		std::cerr << "\033[1;92m[SUCCESS]\033[m Connected to server at " << inet_ntoa(addr) << ':' << ntohs(request.dest_port) << std::endl;
		response.reply = Granted;
		send(sock, &response, 8, 0);

		connect_sockets(sock, server_sock);

		close(server_sock);
		close(sock);
	}

	void oneway_connect (std::string nick, int in, int out) {
		char buf[BUFSIZE];
		memset(buf, 0, BUFSIZE);
		int buflen;
		while ((buflen = recv(in, buf, BUFSIZE, 0)) != 0) {
			send(out, buf, buflen, 0);
			memset(buf, 0, BUFSIZE);
		}
	};

	void Socks4Proxy::connect_sockets(int sock1, int sock2) const {
		std::thread t1(oneway_connect, "client->server", sock1, sock2);
		std::thread t2(oneway_connect, "server->client", sock2, sock1);
		t1.join();
		t2.join();
	}

	void Socks4Proxy::handle_next_connection() const {
		sockaddr_in client_hint;
		socklen_t client_hint_len = sizeof(client_hint);

		auto client_sock = accept(sock, reinterpret_cast<sockaddr*>(&client_hint), &client_hint_len);
		if (client_sock != -1) {
			std::thread([&, client_sock] () { handle_client(client_sock); }).detach();
		}
	}

	Socks4Proxy::Socks4Proxy(int port) {
		sock = socket(AF_INET, SOCK_STREAM, 0);
		{
			int opt = 1;
			setsockopt(sock, SOL_SOCKET, SO_REUSEADDR, &opt, sizeof(opt));
		}

		sockaddr_in hint;
		hint.sin_addr.s_addr = INADDR_ANY;
		hint.sin_port = htons(port);
		hint.sin_family = AF_INET;

		
		if (bind(sock, reinterpret_cast<sockaddr*>(&hint), sizeof(hint)) == -1) {
			throw("Could not bind socket");
		}

		if (listen(sock, 5) == -1) {
			throw("Could not put socket to listen");
		}
	}
}

