project(dataviewer LANGUAGES C CXX)

include(FindPkgConfig)
find_package(Qt5 COMPONENTS Core Quick Multimedia REQUIRED)
find_package(PkgConfig)

set(CMAKE_INCLUDE_CURRENT_DIR ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_CXX_STANDARD 11)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

pkg_check_modules(GLU REQUIRED glu)

add_executable(dataviewer
    main.cpp
    qmlnative/qmlvideohud.cpp
    qmlnative/orientation.cpp
    qml.qrc
)
target_compile_definitions(dataviewer PRIVATE
    $<$<OR:$<CONFIG:Debug>,$<CONFIG:RelWithDebInfo>>:QT_QML_DEBUG>
)
target_link_libraries(dataviewer PRIVATE
    Qt5::Core
    Qt5::Quick
    Qt5::Multimedia
    ${GLU_LIBRARIES}
    cairo
    sensoreval
)
target_include_directories(dataviewer PRIVATE
    ${GLU_INCLUDE_DIRS}
)
target_compile_options(dataviewer PUBLIC
    ${GLU_CFLAGS_OTHER}
)
