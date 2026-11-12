# RAUM

A desktop 3D/4D volume viewer for geophysics data, written in C++20 on Vulkan.

**Status: milestone 0.** The app opens a window, initialises Vulkan 1.3, and clears the
swapchain every frame with a slowly pulsing colour. Nothing is drawn yet. The roadmap
from here is in [docs/PLAN.md](docs/PLAN.md).

## Requirements

- CMake 3.24 or newer, Git, and a C++20 compiler: GCC 12+, Clang 15+, or MSVC 2022.
- The [Vulkan SDK](https://vulkan.lunarg.com/sdk/home) (headers, loader, validation layers).
  A Vulkan 1.3 capable driver is required. On macOS the SDK bundles MoltenVK; use a 2025
  or newer SDK so MoltenVK reports Vulkan 1.3.
- Linux only: the Vulkan loader, validation layers and X11 development packages for GLFW. On Debian/Ubuntu:

  ```bash
  sudo apt install libvulkan-dev vulkan-validationlayers libx11-dev libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev libxext-dev libxkbcommon-dev
  ```

  Native Wayland is off by default (XWayland works). Turn it on with `-DRAUM_GLFW_WAYLAND=ON`
  if you have the Wayland dev packages installed.

Everything else (Vulkan-Headers, GLFW, vk-bootstrap, VulkanMemoryAllocator, GLM) is fetched by
CMake at configure time and built with the project. The Vulkan headers are pinned there too, so
only the loader library and the validation layers come from your SDK. See
`cmake/dependencies.cmake` for the versions.

## Build and run

```bash
cmake -S . -B build -DCMAKE_BUILD_TYPE=Debug
```

```bash
cmake --build build --parallel
```

```bash
./build/src/raum
```

On Windows with Visual Studio the executable lands in `build\src\Debug\raum.exe`.

Debug builds enable the validation layers; Release builds do not. Override with
`--validation` or `--no-validation`. `--frames N` renders N frames and exits, which is
how the smoke test runs. `Esc` closes the window.

## Layout

```
CMakeLists.txt            top-level build
cmake/dependencies.cmake  third-party libraries, pinned by tag
src/main.cpp              argument parsing, error reporting
src/app.*                 composition root and main loop
src/core/                 window (GLFW), Vulkan context, swapchain, logging
src/render/               frame loop: command buffers, sync, acquire/submit/present
docs/PLAN.md              project plan and milestone list
_archive/rust/            the earlier Rust + wgpu prototype this project grew out of
```

## Archive

`_archive/rust` is the original Rust/wgpu prototype, kept intact and buildable with
`cargo run` from inside that directory. It is reference material only; nothing in the
C++ build depends on it.
