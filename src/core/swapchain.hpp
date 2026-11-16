#pragma once

#include <vulkan/vulkan.h>

#include <VkBootstrap.h>

#include <cstdint>
#include <vector>

namespace raum {

class VkContext;

// The swapchain and one image view per swapchain image. Rebuilt on resize.
class Swapchain {
public:
    Swapchain(const VkContext& ctx, VkExtent2D extent);
    ~Swapchain();

    Swapchain(const Swapchain&) = delete;
    Swapchain& operator=(const Swapchain&) = delete;

    // Caller must ensure the device is idle (or at least no frame uses this swapchain).
    void recreate(VkExtent2D extent);

    VkSwapchainKHR handle() const { return swapchain_.swapchain; }
    VkFormat imageFormat() const { return swapchain_.image_format; }
    VkExtent2D extent() const { return swapchain_.extent; }
    uint32_t imageCount() const { return static_cast<uint32_t>(images_.size()); }
    VkImage image(uint32_t index) const { return images_[index]; }
    VkImageView imageView(uint32_t index) const { return imageViews_[index]; }

private:
    void build(VkExtent2D extent);
    void destroyViews();

    const VkContext& ctx_;
    vkb::Swapchain swapchain_{};
    std::vector<VkImage> images_;
    std::vector<VkImageView> imageViews_;
};

}  // namespace raum
