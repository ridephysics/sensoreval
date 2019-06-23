project(main)

pkg_check_modules(GLU REQUIRED glu)

add_executable(sensoreval
    main.cpp
    videohudrenderer.cpp
    sensordata.cpp
    usfsdevice.cpp
    qmlnative/qmlvideohud.cpp
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
)
target_include_directories(sensoreval PRIVATE
    ${GLU_INCLUDE_DIRS}
)
target_compile_options(sensoreval PUBLIC
    ${GLU_CFLAGS_OTHER}
)
