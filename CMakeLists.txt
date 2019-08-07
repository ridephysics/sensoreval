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

add_library(pcg
    ${EXTERNAL_DIR}/pcg-c/src/pcg-advance-8.c
    ${EXTERNAL_DIR}/pcg-c/src/pcg-advance-16.c
    ${EXTERNAL_DIR}/pcg-c/src/pcg-advance-32.c
    ${EXTERNAL_DIR}/pcg-c/src/pcg-advance-64.c
    ${EXTERNAL_DIR}/pcg-c/src/pcg-advance-128.c

    ${EXTERNAL_DIR}/pcg-c/src/pcg-output-8.c
    ${EXTERNAL_DIR}/pcg-c/src/pcg-output-16.c
    ${EXTERNAL_DIR}/pcg-c/src/pcg-output-32.c
    ${EXTERNAL_DIR}/pcg-c/src/pcg-output-64.c
    ${EXTERNAL_DIR}/pcg-c/src/pcg-output-128.c

    ${EXTERNAL_DIR}/pcg-c/src/pcg-rngs-8.c
    ${EXTERNAL_DIR}/pcg-c/src/pcg-rngs-16.c
    ${EXTERNAL_DIR}/pcg-c/src/pcg-rngs-32.c
    ${EXTERNAL_DIR}/pcg-c/src/pcg-rngs-64.c
    ${EXTERNAL_DIR}/pcg-c/src/pcg-rngs-128.c

    ${EXTERNAL_DIR}/pcg-c/src/pcg-global-32.c
    ${EXTERNAL_DIR}/pcg-c/src/pcg-global-64.c
)
target_include_directories(pcg PUBLIC
    ${EXTERNAL_DIR}/pcg-c/include
)

add_subdirectory(lib)
add_subdirectory(hudrenderer)
add_subdirectory(dataviewer)

install(TARGETS
    usfs_calc_quat
    usfs_reader
    dataviewer
    hudrenderer
)
