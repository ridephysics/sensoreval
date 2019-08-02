project(libsensoreval)

add_library(sensoreval
    data.c
    readdata.c
    render.c
    config.c

    hudmodehandlers/booster.c
)
target_include_directories(sensoreval PUBLIC
    include
)
target_link_libraries(sensoreval
    cairo
    m
    cyaml
    yaml
)
