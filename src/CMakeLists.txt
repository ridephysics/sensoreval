project(main)

add_subdirectory(devices)

pkg_check_modules(GLU REQUIRED glu)

add_executable(sensoreval
    main.cpp
    qmlnative/videohud.cpp
    qmlnative/orientation.cpp
    qml.qrc
)
target_compile_definitions(sensoreval PRIVATE
    $<$<OR:$<CONFIG:Debug>,$<CONFIG:RelWithDebInfo>>:QT_QML_DEBUG>
)
target_link_libraries(sensoreval PRIVATE
    Qt5::Core
    Qt5::Quick
    ${GLU_LIBRARIES}
    device_usfs
)
target_include_directories(sensoreval PRIVATE
    ${GLU_INCLUDE_DIRS}
)
target_compile_options(sensoreval PUBLIC
    ${GLU_CFLAGS_OTHER}
)
