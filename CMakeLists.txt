cmake_minimum_required(VERSION 3.1)

project(sensoreval LANGUAGES CXX)

add_subdirectory(${EXTERNAL_DIR}/imu_tools imu_tools)
add_subdirectory(${COMPONENTS_DIR}/usfs usfs)

add_compile_options(
    -Wall
    -Wextra
    -Wshadow
    -Werror
)

include(FindPkgConfig)
find_package(Qt5 COMPONENTS Core Quick REQUIRED)
find_package(PkgConfig)

set(CMAKE_INCLUDE_CURRENT_DIR ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_CXX_STANDARD 11)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

include_directories(include)
add_subdirectory(src)

install(TARGETS
    usfs_calc_quat
    usfs_reader
    sensoreval
)
