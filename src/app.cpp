#include "app.hpp"

#include "core/log.hpp"

#include <cmath>

namespace raum {

App::App(const AppConfig& config)
    : window_(config.width, config.height, config.title),
      ctx_(window_, config.validation),
      swapchain_(ctx_, window_.framebufferExtent()),
      renderer_(window_, ctx_, swapchain_),
      maxFrames_(config.maxFrames) {
    window_.setTitle(config.title + "  |  " + ctx_.deviceName());
}

void App::run() {
    int frames = 0;
    while (!window_.shouldClose()) {
        window_.pollEvents();

        // A slow pulse between two dark blues: proof that frames keep flowing, nothing more.
        const float t = static_cast<float>(glfwGetTime());
        const float pulse = 0.5f + 0.5f * std::sin(t * 0.8f);
        renderer_.setClearColor(0.03f + 0.04f * pulse, 0.05f + 0.08f * pulse, 0.10f + 0.14f * pulse);

        renderer_.drawFrame();

        if (maxFrames_ >= 0 && ++frames >= maxFrames_) {
            log::info("Rendered %d frames, exiting", frames);
            break;
        }
    }
    ctx_.waitIdle();  // nothing may be in flight when the members destruct
}

}  // namespace raum
