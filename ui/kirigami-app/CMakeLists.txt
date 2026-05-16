cmake_minimum_required(VERSION 3.20)
project(ModManagerKirigamiApp LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Qt6 REQUIRED COMPONENTS Core Gui Network Qml Quick)
find_package(KF6 REQUIRED COMPONENTS Kirigami)

qt_standard_project_setup()

qt_add_executable(mod-manager-ui
    src/main.cpp
    src/ipc_client.cpp
    src/ipc_client.h
)

qt_add_qml_module(mod-manager-ui
    URI ModManager
    VERSION 1.0
    QML_FILES
        src/Main.qml
)

target_link_libraries(mod-manager-ui
    PRIVATE
        Qt6::Core
        Qt6::Gui
        Qt6::Network
        Qt6::Qml
        Qt6::Quick
        KF6::Kirigami
)
