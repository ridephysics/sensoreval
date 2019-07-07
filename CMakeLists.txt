cmake_minimum_required(VERSION 3.1)

project(sensoreval)

# technically not needed here but we compile all we need from this top level makefile
add_subdirectory(${EXTERNAL_DIR}/imu_tools imu_tools)
add_subdirectory(${COMPONENTS_DIR}/usfs usfs)

add_compile_options(
    -Wall
    -Wextra
    -Wshadow
    -Werror
)

add_subdirectory(lib)
add_subdirectory(dataviewer)

install(TARGETS
    usfs_calc_quat
    usfs_reader
    dataviewer
)
