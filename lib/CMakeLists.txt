project(libsensoreval)

find_package(PythonLibs REQUIRED)
find_package(PkgConfig)

pkg_check_modules(PANGO REQUIRED IMPORTED_TARGET pango)
pkg_check_modules(PANGOCAIRO REQUIRED IMPORTED_TARGET pangocairo)

add_library(sensoreval
    data.c
    readdata.c
    render.c
    config.c
    plot.c
    math.c

    hudmodehandlers/booster.c
    hudmodehandlers/swingboat.c
)
target_include_directories(sensoreval PUBLIC
    include
    ${PYTHON_INCLUDE_DIRS}
)
target_link_libraries(sensoreval
    pcg
    cairo
    PkgConfig::PANGO
    PkgConfig::PANGOCAIRO
    m
    cyaml
    yaml
    ${PYTHON_LIBRARIES}
)
