project(libsensoreval)

add_library(sensoreval
    renderer.c
)
target_include_directories(sensoreval PUBLIC
    include
)
target_link_libraries(sensoreval
    cairo
)
