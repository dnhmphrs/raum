# RAUM project plan

A desktop viewer for 3D and 4D geophysical volumes (seismic cubes, time-lapse surveys,
inversion results, velocity models) written in C++20 on Vulkan 1.3. It reads HDF5, shows
the volume as slices and as a raymarched render, and draws the line and surface objects
that live alongside a volume: well tracks, horizons, picks, plots.

This document is the working plan. It is meant to be edited as we go: tick things off,
change decisions, add what we learn. Section 9 is the decision log.

---

## 1. Goals and non-goals

**Goals**

- Load a 3D volume (and the 4D case: a sequence of 3D volumes) from HDF5 and look at it
  interactively on a laptop or workstation GPU.
- The interpreter's basic toolkit: orthogonal and oblique slice planes, orbit/pan/zoom
  camera, colormap and clip controls, a raymarched probe with a transfer function,
  isosurface extraction, and picking.
- Auxiliary objects: polylines (wells, picked lines), draped surfaces (horizons), and 2D
  plots (log curves, traces, histograms).
- Clean, explicit engine code that we both understand end to end. No black boxes.

**Non-goals, at least for now**

- Web or mobile builds. Desktop only: Linux, macOS (MoltenVK), Windows.
- Multi-window or multi-GPU.
- A full interpretation suite (attribute computation, horizon autotracking). Those are
  features that could sit on top later; the viewer comes first.
- Supporting every seismic file format. HDF5 is the interchange format; conversion from
  SEG-Y and friends is an external step (a Python script is fine).

## 2. Decisions made

| Area | Decision | Why |
|---|---|---|
| Language | C++20 | Mature Vulkan and HDF5 ecosystems, the libraries below are first-class. |
| Graphics API | Vulkan 1.3, dynamic rendering, synchronization2 | Chosen over WebGPU: full feature set (3D textures, compute, 64-bit atomics later, large resources) and no browser constraints. 1.3 drops render-pass objects and the legacy barrier API. |
| Build | CMake 3.24+, FetchContent for third-party code, versions pinned by tag | One-command build on every platform. |
| Window / input | GLFW 3.4 | Small, stable, creates the Vulkan surface for us. |
| Vulkan setup | vk-bootstrap | Removes ~500 lines of instance/device/swapchain boilerplate without hiding the handles. |
| Vulkan headers | Vulkan-Headers pinned by tag in the build; loader and validation layers from the SDK | Every machine compiles against the same API version whatever SDK it has. vk-bootstrap needs headers at least as new as itself. |
| GPU memory | VulkanMemoryAllocator | Standard answer for buffer and image allocation. |
| Maths | GLM, radians, depth 0..1 | Matches Vulkan clip space. |
| UI | Dear ImGui (docking branch) + ImPlot | Immediate-mode UI is the right fit for a tool with many sliders and panels; ImPlot gives us line plots and histograms for free. |
| Data I/O | HDF5 via HighFive | Header-only C++ wrapper over the HDF5 C library; hyperslab reads map directly onto "give me slice k" and "give me time step t". |
| Shaders | GLSL compiled to SPIR-V by glslc at build time | Readable shaders in the repo, no runtime compiler dependency. |
| Validation | Validation layers on in Debug builds, and a clean run is the bar for every step | Catches synchronisation and lifetime bugs before they become mysteries. |
| Old prototype | Rust/wgpu code lives in `_archive/rust`, untouched | Reference for the camera controller and the raymarch shader ideas; nothing depends on it. |

**Decisions deferred** (see section 7): world coordinate convention, exact HDF5 layout,
whether out-of-core bricking is needed, how 4D is stored.

## 3. Architecture

### 3.1 Modules

```
app                 composition root, main loop, CLI
core/               things every other module borrows
  window            GLFW window, surface, input events
  vk_context        instance, device, queues, VMA allocator
  swapchain         swapchain + views, recreation
  resources         (M1) buffers, images, staging uploads, descriptor helpers
  shaders           (M1) SPIR-V loading, pipeline builders
render/
  renderer          frame loop: acquire, record, submit, present, resize
  frame             (M1) per-frame UBO (camera, time, viewport), depth image
  passes/           (M1+) raster pass, volume pass, ui pass
scene/
  camera            orbit camera and controller (port of the Rust one)
  layer             (M1) interface every drawable thing implements
  layers/           bounding box + grid, slice planes, volume raymarch,
                    polylines, surfaces, isosurface mesh
data/
  volume            (M3) in-memory volume model: dims, spacing, dtype, axes, stats
  hdf5_loader       (M3) HighFive reader, hyperslab access, async loading
  synthetic         (M3) generated test volumes so no file is needed to develop
  colormap          (M4) colormap and transfer-function tables
ui/
  panels            (M2+) ImGui panels: layers, colormap, slices, probe, stats
  plots             (M6) ImPlot views
tools/
  picking           (M6) ray casts, depth readback, pick markers
```

Dependencies point downward: `ui` and `tools` know about `scene`, `scene` knows about
`render` and `data`, everything knows about `core`. Nothing in `core` or `data` includes
Vulkan beyond `core` itself, so `data` stays testable without a GPU.

### 3.2 A frame

```
1. poll input, update camera, write the per-frame UBO (view, proj, inverse, viewport, time)
2. raster pass   colour + depth, dynamic rendering
                 grid and bounding box, slice quads, horizons, wells, pick markers
3. volume pass   fullscreen triangle; reads the depth image as a texture and the 3D
                 volume texture; raymarch stops where it hits raster geometry;
                 blends over the colour image
4. ui pass       ImGui, no depth
5. present
```

Everything rasterised shares one depth buffer, and the raymarch respects it. That gives
correct occlusion between the volume and lines/surfaces without any sorting, and lets
the volume pass be switched off entirely for cheap slice-only viewing.

### 3.3 Data flow for a volume

```
HDF5 file --(hyperslab read, worker thread)--> Volume (CPU, typed, with stats)
       --(staging buffer upload)--> VkImage 3D, R8/R16/R32F --(sampler)--> shaders
Transfer function / colormap (CPU table) --> 1D RGBA8 texture --> shaders
```

The CPU `Volume` keeps the value range and a histogram so percentile clipping and the
colormap editor work without touching the GPU copy.

## 4. Conventions

- **Code**: C++20, no exceptions across module boundaries except fatal setup errors,
  RAII for GPU objects, `namespace raum`, `snake_case` files, `PascalCase` types,
  `camelCase` functions and members with a trailing underscore. Warnings as the
  compiler's `-Wall -Wextra -Wpedantic` and we keep them at zero.
- **Vulkan**: synchronization2 for every barrier and submit; dynamic rendering, never
  `VkRenderPass`; VMA for every allocation; `vkCheck()` on every call that can fail.
- **Descriptor sets**: set 0 is per-frame globals (camera UBO, time), set 1 is per-layer
  (volume texture, colormap, layer parameters), push constants for per-draw data.
- **Shaders**: GLSL 4.6 in `shaders/`, compiled by CMake into `build/shaders/*.spv` next
  to the executable. Shared struct layouts live in a `.glsl` include that mirrors the
  C++ header, and we keep the two in step by hand.
- **Coordinates**: right-handed world space in metres (or samples when the file gives no
  spacing). Proposed mapping, to confirm in section 7: x = inline direction, y =
  crossline direction, z = depth or two-way time with **depth increasing downward along
  negative z**, so "up" in the viewer is the surface. The `Volume` carries an origin and
  per-axis spacing, and layers transform through one `volumeToWorld` matrix.
- **Colour**: swapchain is sRGB; all shader colour math happens in linear space.
- **Testing**: `raum --frames N` must exit cleanly with zero validation messages after
  every change. Synthetic volumes for development, so a missing data file never blocks
  work. CPU-side code (data model, loaders, colormaps) gets unit tests from M3 on.

## 5. Milestones

Sizes are rough and assume a few focused sessions per unit of work. Each milestone ends
with something you can run and see.

### M0. Window and clear (done 2026-11-12)

- CMake project, pinned dependencies, Debug/Release configs.
- GLFW window, Vulkan 1.3 instance with validation, device selection, swapchain,
  two frames in flight, resize and minimise handling, dynamic-rendering clear.
- `--frames N` smoke test.
- **Acceptance**: window opens on all three platforms, colour pulses, resizing works, zero
  validation messages, `raum --frames 60` exits 0.

### M1. First geometry: bounding box, grid, orbit camera

Goal: the empty "room" a volume will sit in, with a camera you can fly.

1. Shader build step in CMake (`glslc`), a SPIR-V loader, and a small graphics pipeline
   builder (dynamic rendering, vertex layout, depth test, blend mode, topology).
2. `resources`: VMA-backed buffer and image wrappers, a staging upload helper, a
   descriptor-set-layout builder.
3. Depth image (D32_SFLOAT) created with the swapchain and cleared each frame.
4. Per-frame UBO with camera matrices and time; descriptor set 0.
5. Orbit camera and controller: left-drag orbit, right-drag or middle-drag pan, scroll
   zoom, keyboard nudges. Port the logic from `_archive/rust/src/camera/controller.rs`.
6. `Layer` interface (`update`, `record(cmd, frame)`, `ui()` later) and a `LayerStack`.
7. First layers: a wireframe bounding box and a ground grid as line lists.
- **Acceptance**: a cube and a grid you can orbit around, correct depth, correct aspect
  on resize, still zero validation messages.
- **Size**: about a week.

### M2. UI and input routing

1. Dear ImGui (docking) with the GLFW and Vulkan backends; UI pass after the raster pass.
2. Input goes to ImGui first, the camera second.
3. A stats overlay (frame time, GPU name, swapchain size) and a "Layers" panel that lists
   layers with visibility toggles.
4. Colour-correct ImGui on an sRGB swapchain.
- **Acceptance**: panels dock and resize, camera does not move when dragging a slider.
- **Size**: two or three days.

### M3. Data model and HDF5 loading

1. `Volume`: dimensions, dtype (int8, int16, float32), axis names, origin and spacing,
   value range, histogram, percentile lookup. Owns its memory.
2. `synthetic`: generated volumes (layered reflectors with a fault, a sphere, noise) so
   every later milestone can be developed without a file.
3. HighFive + the HDF5 C library in the build (system package or vcpkg; document both).
4. `hdf5_loader`: open a file, list datasets, read a whole 3D dataset, read one 3D step
   out of a 4D dataset with a hyperslab, read attributes for spacing and axis names.
5. CLI: `raum path/to/file.h5[:dataset]`, and a File menu later.
6. Unit tests for `Volume` and the loader against a tiny HDF5 file generated in-test.
- **Acceptance**: `raum --info file.h5` prints dims, dtype, range and percentiles.
- **Size**: three to five days. Agreeing on the HDF5 layout (section 7) comes first.

### M4. Slices

Goal: the workhorse view. Most interpretation time is spent here.

1. Upload the volume to a 3D texture (R8_UNORM, R16_SNORM or R32_SFLOAT by dtype) via a
   staging buffer; normalisation range as a uniform.
2. Colormap module: built-in maps (seismic red-white-blue, greyscale, viridis-like),
   uploaded as a 256-entry 1D texture; clip range from percentiles.
3. `SliceLayer`: three axis-aligned quads sampling the 3D texture, drawn with depth.
   Sliders and keyboard nudging for inline / crossline / depth position.
4. Oblique slice: an arbitrary plane clipped to the volume box.
5. Colormap panel: map picker, clip range with a histogram behind it, gain, polarity flip.
- **Acceptance**: a synthetic cube and a real file look right, slices move at 60 fps.
- **Size**: about a week.

### M5. Volume raymarching

1. Volume pass: fullscreen triangle, ray/box intersection in volume space, front-to-back
   compositing with early termination, transfer function texture (RGBA, alpha = opacity).
2. Depth-aware termination: read the raster depth image, reconstruct the world position,
   stop the ray there.
3. Probe mode: a movable sub-box; raymarch only inside it while slices show the rest.
4. Modes: composite, maximum intensity projection, average.
5. Gradient-based shading (central differences) with a toggle.
6. Quality controls: step size, jitter to hide banding, half-resolution option.
7. Transfer-function editor in the UI.
- **Acceptance**: probe renders over slices with correct occlusion against wells and
  slices; interactive on a mid-range GPU at full window size.
- **Size**: one to two weeks.

### M6. Line objects, surfaces, picking, plots

1. `PolylineLayer`: polylines with per-vertex colour, widths via screen-space quads,
   depth tested. Well tracks are the first client.
2. `SurfaceLayer`: triangulated horizons, draped with an attribute colormap, optional
   wireframe.
3. Isosurface extraction with marching cubes (CPU first, compute shader if it matters).
4. Picking: ray cast against slice planes and the volume box on the CPU; depth readback
   for lines and surfaces; pick markers as a layer.
5. ImPlot views: trace at the picked position, histogram, log curve alongside a well.
6. Loading these objects: simple CSV/JSON for now, HDF5 groups later.
- **Acceptance**: pick a point on a slice, see its trace; wells and a horizon render
  correctly inside and outside the probe.
- **Size**: one to two weeks.

### M7. 4D

1. Define the 4D convention in the file (section 7) and in `Volume` (a step index).
2. Worker thread that reads step t+1 while t is on screen; double-buffered 3D textures.
3. Time-step slider, playback with a rate, difference mode (t minus baseline).
- **Acceptance**: scrub through a synthetic 4D cube without stalls; playback is smooth
  for volumes that fit in RAM.
- **Size**: one week if the volumes fit in memory.

### M8. Large data (only if needed)

Triggered when a target volume does not fit in VRAM.

1. Mipmapped 3D texture for level-of-detail during interaction.
2. Brick cache: the volume split into fixed-size bricks, a resident set chosen from the
   view, loaded on demand from chunked HDF5.
3. Out-of-core slices: read only the hyperslab a slice needs.
- **Size**: three to six weeks. This is the biggest fork in the road and we decide it
  with real file sizes in hand.

### M9. Polish

Screenshots to PNG, session save/load (camera, layers, colormaps), settings file,
packaging (AppImage, dmg, zip), CI that builds all three platforms and runs the smoke
test on a software Vulkan driver (lavapipe), a short user guide.

## 6. HDF5 data model (proposal)

A volume is one dataset. Dimension order on disk is `[z][y][x]` (slowest to fastest), or
`[t][z][y][x]` for 4D. Supported dtypes: int8, int16, float32. Attributes on the dataset:

| Attribute | Type | Meaning |
|---|---|---|
| `axes` | string[3 or 4] | names, e.g. `["twt", "crossline", "inline"]` or `["time", "depth", "y", "x"]` |
| `origin` | float64[3] | world coordinate of sample (0,0,0) |
| `spacing` | float64[3] | sample interval per axis, in the units below |
| `units` | string[3] | e.g. `["ms", "m", "m"]` |
| `value_range` | float64[2] | optional; computed if missing |
| `null_value` | scalar | optional; samples to treat as missing |

Chunking: `[1][64][64][64]`-ish chunks make both slice reads and 4D step reads cheap.
The loader must still cope with unchunked, attribute-less datasets by falling back to
sample units and a computed range. A `tools/` Python script that converts SEG-Y to this
layout is a small, separate job.

## 7. Open questions for Dan

Answers to these change the plan, so they come before the milestone they affect.

1. **Typical and maximum volume sizes** (samples per axis, dtype). Decides whether M8
   exists and whether int16 vs float32 is the default upload format. *(Needed by M4.)*
2. **What the fourth dimension is** in your data: time-lapse surveys, pre-stack offset or
   angle gathers, Monte-Carlo realisations, something else? Decides the 4D UI and whether
   "difference from baseline" is a mode. *(Needed by M7.)*
3. **Existing HDF5 files**: do you already have a layout, or shall we adopt section 6?
   *(Needed by M3.)*
4. **Coordinate convention**: is the proposed mapping in section 4 right for you, and is
   depth-down with the surface as "up" the view you want? *(Needed by M1.)*
5. **Target machines**: which OS and GPUs you and the eventual users have. Decides how
   hard we chase MoltenVK quirks and whether half-resolution raymarching is a default.
6. **Line object sources**: where wells, horizons and picks will come from (files, a
   database, another app). Decides M6's loaders.

## 8. How we work

- One milestone step per sitting where possible; each ends with a commit that builds and
  passes the smoke test.
- Validation-clean is a requirement, not a goal.
- When a decision is made, it goes into section 9 with a one-line reason. When a step is
  done, it gets a date in section 5.
- New third-party code only via `cmake/dependencies.cmake`, pinned to a tag, with a line
  in the table in section 2 saying why it is there.
- Keep `_archive/rust` as is.

## 9. Decision log

| Date | Decision |
|---|---|
| 2025-12 | Switch the project from Rust/wgpu to C++/Vulkan. |
| 2026-11-12 | Archive the Rust prototype under `_archive/rust`; start the C++ tree. |
| 2026-11-12 | Vulkan 1.3 minimum, with dynamic rendering and synchronization2 required at device selection. |
| 2026-11-12 | vk-bootstrap, VMA, GLFW 3.4 and GLM via FetchContent; HDF5 from the system, HighFive via FetchContent (to do in M3). |
| 2026-11-12 | Render-finished semaphores are per swapchain image, not per frame in flight. |
| 2026-11-12 | Vulkan-Headers pinned in the build (same tag as vk-bootstrap); only the loader library is taken from the SDK. Ubuntu 24.04's packaged 1.3.275 headers were too old for vk-bootstrap 1.4. |
