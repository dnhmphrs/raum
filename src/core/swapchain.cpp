#include "core/swapchain.hpp"

#include "core/log.hpp"
#include "core/vk_context.hpp"

#include <stdexcept>

namespace raum {

Swapchain::Swapchain(const VkContext& ctx, VkExtent2D extent) : ctx_(ctx) {
    build(extent);
}

Swapchain::~Swapchain() {
    destroyViews();
    if (swapchain_.swapchain != VK_NULL_HANDLE) {
        vkb::destroy_swapchain(swapchain_);
    }
}

void Swapchain::recreate(VkExtent2D extent) {
    destroyViews();
    build(extent);
}

void Swapchain::build(VkExtent2D extent) {
    vkb::SwapchainBuilder builder{ctx_.vkbDevice()};
    builder.set_desired_format(VkSurfaceFormatKHR{VK_FORMAT_B8G8R8A8_SRGB, VK_COLOR_SPACE_SRGB_NONLINEAR_KHR})
        .set_desired_present_mode(VK_PRESENT_MODE_FIFO_KHR)  // vsync; guaranteed to exist
        .set_desired_extent(extent.width, extent.height)
        .add_image_usage_flags(VK_IMAGE_USAGE_TRANSFER_DST_BIT)
        .set_old_swapchain(swapchain_);

    auto ret = builder.build();
    if (!ret) {
        throw std::runtime_error("Failed to create swapchain: " + ret.error().message());
    }

    // The new swapchain has taken over the old one's resources; now release the old handle.
    if (swapchain_.swapchain != VK_NULL_HANDLE) {
        vkb::destroy_swapchain(swapchain_);
    }
    swapchain_ = ret.value();

    auto imagesRet = swapchain_.get_images();
    auto viewsRet = swapchain_.get_image_views();
    if (!imagesRet || !viewsRet) {
        throw std::runtime_error("Failed to get swapchain images/views");
    }
    images_ = imagesRet.value();
    imageViews_ = viewsRet.value();

    log::info("Swapchain: %ux%u, %u images, format %d",
              swapchain_.extent.width, swapchain_.extent.height, imageCount(),
              static_cast<int>(swapchain_.image_format));
}

void Swapchain::destroyViews() {
    if (!imageViews_.empty()) {
        swapchain_.destroy_image_views(imageViews_);
        imageViews_.clear();
    }
    images_.clear();
}

}  // namespace raum
