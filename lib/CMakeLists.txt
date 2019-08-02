project(libsensoreval)

find_package(PythonLibs REQUIRED)

add_library(sensoreval
    data.c
    readdata.c
    render.c
    config.c
    plot.c

    hudmodehandlers/booster.c
)
target_include_directories(sensoreval PUBLIC
    include
    ${PYTHON_INCLUDE_DIRS}
)
target_link_libraries(sensoreval
    cairo
    m
    cyaml
    yaml
    ${PYTHON_LIBRARIES}
)
