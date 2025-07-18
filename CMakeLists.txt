cmake_minimum_required(VERSION 3.31)
project(fervorous)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)


# Engine
add_subdirectory("vendor/glfw-3.4")
add_subdirectory("vendor/glad")

add_library(engine STATIC)
target_link_libraries(engine PRIVATE glfw glad)
target_sources(engine
        PUBLIC
        FILE_SET CXX_MODULES FILES
        "engine/opengl.cc"
)

# Core
add_executable(core "core/main.cc")
target_link_libraries(core PRIVATE engine)