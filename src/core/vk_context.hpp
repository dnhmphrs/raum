#pragma once

#include <vulkan/vulkan.h>

#include <VkBootstrap.h>
#include <vk_mem_alloc.h>

#include <cstdint>
#include <string>

namespace raum {

class Window;

// Owns the Vulkan instance, debug messenger, surface, physical and logical device,
// the queues we use, and the VMA allocator. Everything else in the app borrows these.
class VkContext {
public:
    VkContext(const Window& window, bool enableValidation);
    ~VkContext();

    VkContext(const VkContext&) = delete;
    VkContext& operator=(const VkContext&) = delete;

    VkInstance instance() const { return instance_.instance; }
    VkPhysicalDevice physicalDevice() const { return physicalDevice_.physical_device; }
    VkDevice device() const { return device_.device; }
    VkSurfaceKHR surface() const { return surface_; }
    VkQueue graphicsQueue() const { return graphicsQueue_; }
    uint32_t graphicsQueueFamily() const { return graphicsQueueFamily_; }
    VkQueue presentQueue() const { return presentQueue_; }
    VmaAllocator allocator() const { return allocator_; }

    const vkb::Device& vkbDevice() const { return device_; }
    std::string deviceName() const { return physicalDevice_.properties.deviceName; }

    void waitIdle() const;

private:
    vkb::Instance instance_{};
    VkSurfaceKHR surface_ = VK_NULL_HANDLE;
    vkb::PhysicalDevice physicalDevice_{};
    vkb::Device device_{};
    VkQueue graphicsQueue_ = VK_NULL_HANDLE;
    uint32_t graphicsQueueFamily_ = 0;
    VkQueue presentQueue_ = VK_NULL_HANDLE;
    VmaAllocator allocator_ = VK_NULL_HANDLE;
};

}  // namespace raum
