# Third-party dependencies, fetched at configure time and built as part of this project.
# Versions are pinned to tags so a build is reproducible.

include(FetchContent)

# --- Vulkan headers + loader ------------------------------------------------------------
# The headers are pinned here (same tag as vk-bootstrap, which needs a matching version)
# so every machine compiles against the same Vulkan API version regardless of which SDK
# it has installed. The loader library and validation layers still come from the SDK.
set(VULKAN_HEADERS_ENABLE_MODULE OFF CACHE BOOL "" FORCE)
set(VULKAN_HEADERS_ENABLE_INSTALL OFF CACHE BOOL "" FORCE)
FetchContent_Declare(VulkanHeaders
  GIT_REPOSITORY https://github.com/KhronosGroup/Vulkan-Headers.git
  GIT_TAG        v1.4.367
  GIT_SHALLOW    TRUE)
FetchContent_MakeAvailable(VulkanHeaders)   # defines Vulkan::Headers

# Must come after the headers so FindVulkan does not define its own Vulkan::Headers.
find_package(Vulkan)
if(NOT Vulkan_FOUND)
  message(FATAL_ERROR
    "Vulkan SDK not found. Install it from https://vulkan.lunarg.com/sdk/home, then open a NEW "
    "terminal so the VULKAN_SDK environment variable is picked up, and configure again. "
    "On Linux you can instead install the distro packages (libvulkan-dev, vulkan-validationlayers). "
    "Only the loader library and validation layers are taken from the SDK; headers are pinned above.")
endif()
set(RAUM_VULKAN_LOADER ${Vulkan_LIBRARIES})

# --- GLFW: window, input, Vulkan surface creation --------------------------------------
set(GLFW_BUILD_DOCS OFF CACHE BOOL "" FORCE)
set(GLFW_BUILD_TESTS OFF CACHE BOOL "" FORCE)
set(GLFW_BUILD_EXAMPLES OFF CACHE BOOL "" FORCE)
set(GLFW_INSTALL OFF CACHE BOOL "" FORCE)
if(UNIX AND NOT APPLE)
  set(GLFW_BUILD_X11 ON CACHE BOOL "" FORCE)
  set(GLFW_BUILD_WAYLAND ${RAUM_GLFW_WAYLAND} CACHE BOOL "" FORCE)
endif()
FetchContent_Declare(glfw
  GIT_REPOSITORY https://github.com/glfw/glfw.git
  GIT_TAG        3.4
  GIT_SHALLOW    TRUE)

# --- vk-bootstrap: instance / device / swapchain creation without the boilerplate -------
set(VK_BOOTSTRAP_TEST OFF CACHE BOOL "" FORCE)
set(VK_BOOTSTRAP_WERROR OFF CACHE BOOL "" FORCE)
FetchContent_Declare(vk-bootstrap
  GIT_REPOSITORY https://github.com/charles-lunarg/vk-bootstrap.git
  GIT_TAG        v1.4.367
  GIT_SHALLOW    TRUE)

# --- VulkanMemoryAllocator: buffer and image memory management -------------------------
FetchContent_Declare(VulkanMemoryAllocator
  GIT_REPOSITORY https://github.com/GPUOpen-LibrariesAndSDKs/VulkanMemoryAllocator.git
  GIT_TAG        v3.5.0
  GIT_SHALLOW    TRUE)

# --- GLM: vector / matrix maths -----------------------------------------------------------
set(GLM_BUILD_TESTS OFF CACHE BOOL "" FORCE)
set(GLM_BUILD_INSTALL OFF CACHE BOOL "" FORCE)
FetchContent_Declare(glm
  GIT_REPOSITORY https://github.com/g-truc/glm.git
  GIT_TAG        1.0.1
  GIT_SHALLOW    TRUE)

FetchContent_MakeAvailable(glfw vk-bootstrap VulkanMemoryAllocator glm)
