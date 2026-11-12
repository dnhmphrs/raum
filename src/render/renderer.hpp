#pragma once

#include <vulkan/vulkan.h>

#include <array>
#include <cstdint>
#include <vector>

namespace raum {

class Window;
class VkContext;
class Swapchain;

// Drives the frame loop: per-frame command buffers and sync objects, acquire / record /
// submit / present, and swapchain recreation on resize. Right now the only thing it
// records is a clear; later milestones add render passes here (or move them out into
// a layer system that this class calls into).
class Renderer {
public:
    Renderer(Window& window, VkContext& ctx, Swapchain& swapchain);
    ~Renderer();

    Renderer(const Renderer&) = delete;
    Renderer& operator=(const Renderer&) = delete;

    void drawFrame();

    void setClearColor(float r, float g, float b);

private:
    static constexpr uint32_t kFramesInFlight = 2;

    struct FrameData {
        VkCommandPool commandPool = VK_NULL_HANDLE;
        VkCommandBuffer commandBuffer = VK_NULL_HANDLE;
        VkSemaphore imageAvailable = VK_NULL_HANDLE;  // swapchain image acquired
        VkFence inFlight = VK_NULL_HANDLE;            // this frame's GPU work finished
    };

    void createFrameData();
    void destroyFrameData();
    void createRenderFinishedSemaphores();
    void destroyRenderFinishedSemaphores();
    void recreateSwapchain();
    void recordCommands(VkCommandBuffer cmd, uint32_t imageIndex);

    Window& window_;
    VkContext& ctx_;
    Swapchain& swapchain_;

    std::array<FrameData, kFramesInFlight> frames_{};
    // One per swapchain image (not per frame): a present may still be waiting on it
    // when the same frame slot comes around again.
    std::vector<VkSemaphore> renderFinished_;
    uint32_t frameIndex_ = 0;

    VkClearColorValue clearColor_{};
};

}  // namespace raum
