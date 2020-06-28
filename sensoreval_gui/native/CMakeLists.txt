cmake_minimum_required(VERSION 3.1)

project(sensorevalgui_native LANGUAGES C CXX)

include(FindPkgConfig)
find_package(Qt5 COMPONENTS Core Quick Multimedia REQUIRED)
find_package(PkgConfig)

set(CMAKE_INCLUDE_CURRENT_DIR ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_CXX_STANDARD 11)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

pkg_check_modules(GLU REQUIRED glu)

add_compile_options(
    -Wall
    -Wextra
    -Wshadow
    -Werror

    -fno-exceptions
)

add_library(sensorevalgui_native
    main.cpp
    qmlnative/qmlvideohud.cpp
    qmlnative/orientation.cpp
    qml.qrc
)
target_compile_definitions(sensorevalgui_native PRIVATE
    $<$<OR:$<CONFIG:Debug>,$<CONFIG:RelWithDebInfo>>:QT_QML_DEBUG>
)
target_link_libraries(sensorevalgui_native PRIVATE
    Qt5::Core
    Qt5::Quick
    Qt5::Multimedia
    ${GLU_LIBRARIES}
    cairo
)
target_include_directories(sensorevalgui_native PRIVATE
    ${GLU_INCLUDE_DIRS}
    include
)
target_compile_options(sensorevalgui_native PUBLIC
    ${GLU_CFLAGS_OTHER}
)

install(TARGETS
    sensorevalgui_native
)