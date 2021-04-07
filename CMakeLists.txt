cmake_minimum_required(VERSION 3.5)

project(webexp)
set(CMAKE_CXX_STANDARD 20)

add_executable(webexp src/main.cpp src/http/request.cpp src/proxy/socks4.cpp)

target_link_libraries(webexp pthread)

