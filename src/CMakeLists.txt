add_executable(raum
  main.cpp
  app.cpp
  core/log.cpp
  core/window.cpp
  core/vk_context.cpp
  core/swapchain.cpp
  core/vma_impl.cpp
  render/renderer.cpp
)

target_include_directories(raum PRIVATE ${CMAKE_CURRENT_SOURCE_DIR})

target_link_libraries(raum PRIVATE
  Vulkan::Headers
  ${RAUM_VULKAN_LOADER}
  glfw
  vk-bootstrap::vk-bootstrap
  GPUOpen::VulkanMemoryAllocator
  glm::glm
)

target_compile_definitions(raum PRIVATE
  GLFW_INCLUDE_NONE             # we include vulkan.h ourselves; no OpenGL headers
  GLM_FORCE_RADIANS
  GLM_FORCE_DEPTH_ZERO_TO_ONE   # Vulkan clip space is [0, 1] in depth
  GLM_ENABLE_EXPERIMENTAL
)

if(MSVC)
  target_compile_options(raum PRIVATE /W4 /permissive-)
  target_compile_definitions(raum PRIVATE NOMINMAX WIN32_LEAN_AND_MEAN)
else()
  target_compile_options(raum PRIVATE -Wall -Wextra -Wpedantic)
endif()

# VMA's implementation unit is third-party code; don't let its warnings drown ours.
set_source_files_properties(core/vma_impl.cpp PROPERTIES COMPILE_OPTIONS "$<IF:$<CXX_COMPILER_ID:MSVC>,/w,-w>")
