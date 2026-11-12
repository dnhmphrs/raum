cmake_minimum_required(VERSION 3.24)

project(raum
  VERSION 0.1.0
  DESCRIPTION "3D/4D volume viewer for geophysics data (C++20 / Vulkan)"
  LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# Default to a Debug build (validation layers on) for single-config generators.
if(NOT CMAKE_BUILD_TYPE AND NOT CMAKE_CONFIGURATION_TYPES)
  set(CMAKE_BUILD_TYPE Debug CACHE STRING "Build type" FORCE)
  set_property(CACHE CMAKE_BUILD_TYPE PROPERTY STRINGS Debug Release RelWithDebInfo)
endif()

option(RAUM_GLFW_WAYLAND "Build GLFW with native Wayland support (needs wayland dev packages; X11/XWayland works without it)" OFF)

# Vulkan headers are pinned in cmake/dependencies.cmake; the SDK supplies the loader
# library we link and the validation layers we use at runtime.
include(cmake/dependencies.cmake)

add_subdirectory(src)
