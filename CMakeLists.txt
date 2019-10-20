cmake_minimum_required(VERSION 3.1)

project(sensoreval)
list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/cmake")

# technically not needed here but we compile all we need from this top level makefile
add_subdirectory(${EXTERNAL_DIR}/imu_tools imu_tools)
add_subdirectory(${COMPONENTS_DIR}/usfs usfs)

# lookup dependencies
include(FeatureSummary)
include(GNUInstallDirs)
find_package(Cargo REQUIRED)
find_package(Rust REQUIRED)
set(CMAKE_THREAD_PREFER_PTHREAD TRUE)
find_package(Threads REQUIRED)
find_package (PkgConfig REQUIRED)
find_library(DL_LIBRARY ${CMAKE_DL_LIBS})
pkg_check_modules(GLIB REQUIRED glib-2.0)
pkg_check_modules(GOBJECT REQUIRED gobject-2.0)
pkg_check_modules(CAIRO_GOBJECT REQUIRED cairo-gobject)
pkg_check_modules(PANGO REQUIRED pango)
pkg_check_modules(PANGOCAIRO REQUIRED pangocairo)

# setup variables for rust
set(RUST_SRC "${CMAKE_CURRENT_SOURCE_DIR}/src")
set(RUST_OUT "${CMAKE_BINARY_DIR}/rust")
file(GLOB_RECURSE RUST_DEPS ${RUST_SRC}/* ${CMAKE_CURRENT_SOURCE_DIR}/Cargo.*)
set(RUST_BUILD_FLAGS --target-dir ${RUST_OUT} --color always)
string(TOUPPER "${CMAKE_BUILD_TYPE}" CMAKE_BUILD_TYPE_UPPER)
if(CMAKE_BUILD_TYPE_UPPER STREQUAL DEBUG)
    set(RUST_TARGET_DIR ${RUST_OUT}/debug)
else()
    set(RUST_TARGET_DIR ${RUST_OUT}/release)
    set(RUST_BUILD_FLAGS ${RUST_BUILD_FLAGS} --release)
endif()
set(RUST_LIB "${RUST_TARGET_DIR}/libsensoreval.a")

# rust target
add_custom_command(
  OUTPUT "${RUST_LIB}"
  DEPENDS ${RUST_DEPS}
  COMMAND ${Cargo_EXECUTABLE} build ${RUST_BUILD_FLAGS}
  WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
)
add_custom_target(sensoreval_lib DEPENDS "${RUST_LIB}")

# rust library
add_library(sensoreval STATIC IMPORTED)
add_dependencies(sensoreval sensoreval_lib)
set_target_properties(sensoreval PROPERTIES IMPORTED_LOCATION "${RUST_LIB}")
target_include_directories(sensoreval INTERFACE ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(sensoreval INTERFACE
    ${CMAKE_DL_LIBS}
    ${CMAKE_THREAD_LIBS_INIT}
    ${GLIB_LIBRARIES}
    ${GOBJECT_LIBRARIES}
    ${CAIRO_GOBJECT_LIBRARIES}
    ${PANGO_LIBRARIES}
    ${PANGOCAIRO_LIBRARIES}
)

add_compile_options(
    -Wall
    -Wextra
    -Wshadow
    -Werror
)

add_subdirectory(dataviewer)

install(TARGETS
    usfs_calc_quat
    usfs_reader
    dataviewer
)

function(paths_to_names var)
    set(macro_tmp)
    foreach(s ${${var}})
        get_filename_component(s ${s} NAME)
        string(REGEX REPLACE "\\.[^.]*$" "" s ${s})
        list(APPEND macro_tmp "${s}")
    endforeach()

    set(${var} "${macro_tmp}" PARENT_SCOPE)
endfunction()

function(list_prepend var prep)
    set(macro_tmp)
    foreach(s ${${var}})
        list(APPEND macro_tmp "${prep}${s}")
    endforeach()

    set(${var} "${macro_tmp}" PARENT_SCOPE)
endfunction()

# install all rust binaries
file(GLOB RUST_BINS ${RUST_SRC}/bin/*.rs)
paths_to_names(RUST_BINS)
list_prepend(RUST_BINS "${RUST_TARGET_DIR}/")
message("${CMAKE_INSTALL_BINDIR}")
install(PROGRAMS ${RUST_BINS} DESTINATION "${CMAKE_INSTALL_BINDIR}")
