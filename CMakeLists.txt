cmake_minimum_required(VERSION 3.1)

project(sensoreval LANGUAGES CXX)

include(FindPkgConfig)
find_package(Qt5 COMPONENTS Core Quick REQUIRED)
find_package(PkgConfig)

set(CMAKE_INCLUDE_CURRENT_DIR ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_CXX_STANDARD 11)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

include_directories(include)

add_subdirectory(${COMPONENTS_DIR}/usfs usfs)
add_subdirectory(src)
