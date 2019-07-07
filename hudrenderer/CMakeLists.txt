project(hudrenderer)

add_executable(hudrenderer
    main.c
)
target_link_libraries(hudrenderer
    sensoreval
    cairo
)
