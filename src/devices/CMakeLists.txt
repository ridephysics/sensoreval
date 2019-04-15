project(devices)

add_library(device_usfs STATIC
    usfsdevice.cpp
)
target_link_libraries(device_usfs PUBLIC
    Qt5::Core
    Qt5::Gui
    em7180
    crossi2c
)
