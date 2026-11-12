#pragma once

#include "core/swapchain.hpp"
#include "core/vk_context.hpp"
#include "core/window.hpp"
#include "render/renderer.hpp"

#include <string>

namespace raum {

struct AppConfig {
    int width = 1280;
    int height = 800;
    std::string title = "RAUM";
    bool validation = true;
    int maxFrames = -1;  // >= 0: exit after this many frames (smoke tests, CI)
};

// Composition root. Member order is construction order, and destruction runs in reverse,
// so the renderer goes before the swapchain, before the device, before the window.
class App {
public:
    explicit App(const AppConfig& config);

    void run();

private:
    Window window_;
    VkContext ctx_;
    Swapchain swapchain_;
    Renderer renderer_;
    int maxFrames_;
};

}  // namespace raum
