project(libsensoreval)

add_library(sensoreval
    data.c
    render.c
)
target_include_directories(sensoreval PUBLIC
    include
)
target_link_libraries(sensoreval
    cairo
    m
)
