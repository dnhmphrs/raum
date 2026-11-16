#include "core/vk_context.hpp"

#include "core/log.hpp"
#include "core/vk_check.hpp"
#include "core/window.hpp"

#include <stdexcept>

namespace raum {

VkContext::VkContext(const Window& window, bool enableValidation) {
    // --- Instance ------------------------------------------------------------------------
    auto systemInfoRet = vkb::SystemInfo::get_system_info();
    if (!systemInfoRet) {
        throw std::runtime_error("Failed to query Vulkan system info: " + systemInfoRet.error().message());
    }
    const auto& systemInfo = systemInfoRet.value();

    const bool useValidation = enableValidation && systemInfo.validation_layers_available;
    if (enableValidation && !useValidation) {
        log::warn("Validation layers requested but VK_LAYER_KHRONOS_validation is not installed; continuing without them");
    }

    vkb::InstanceBuilder instanceBuilder;
    instanceBuilder.set_app_name("raum")
        .set_engine_name("raum")
        .require_api_version(1, 3, 0);
    if (useValidation) {
        // vk-bootstrap's default messenger prints warnings and errors to stdout.
        instanceBuilder.request_validation_layers(true).use_default_debug_messenger();
    }
    // vk-bootstrap enables VK_KHR_surface and the platform surface extension(s) itself,
    // and VK_KHR_portability_enumeration on MoltenVK, so GLFW's list is not needed here.
    auto instanceRet = instanceBuilder.build();
    if (!instanceRet) {
        throw std::runtime_error("Failed to create Vulkan instance: " + instanceRet.error().message());
    }
    instance_ = instanceRet.value();

    // --- Surface -------------------------------------------------------------------------
    surface_ = window.createSurface(instance_.instance);

    // --- Physical device -----------------------------------------------------------------
    // Features we build on from day one: dynamic rendering (no render pass objects) and
    // synchronization2 (the newer, clearer barrier API). Both are core in Vulkan 1.3.
    VkPhysicalDeviceVulkan13Features features13{};
    features13.sType = VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_VULKAN_1_3_FEATURES;
    features13.dynamicRendering = VK_TRUE;
    features13.synchronization2 = VK_TRUE;

    vkb::PhysicalDeviceSelector selector{instance_};
    auto physicalRet = selector.set_surface(surface_)
                           .set_minimum_version(1, 3)
                           .set_required_features_13(features13)
                           .select();
    if (!physicalRet) {
        throw std::runtime_error("No suitable Vulkan 1.3 GPU found: " + physicalRet.error().message());
    }
    physicalDevice_ = physicalRet.value();

    // --- Logical device and queues -------------------------------------------------------
    vkb::DeviceBuilder deviceBuilder{physicalDevice_};
    auto deviceRet = deviceBuilder.build();
    if (!deviceRet) {
        throw std::runtime_error("Failed to create Vulkan device: " + deviceRet.error().message());
    }
    device_ = deviceRet.value();

    auto graphicsQueueRet = device_.get_queue(vkb::QueueType::graphics);
    auto graphicsFamilyRet = device_.get_queue_index(vkb::QueueType::graphics);
    auto presentQueueRet = device_.get_queue(vkb::QueueType::present);
    if (!graphicsQueueRet || !graphicsFamilyRet || !presentQueueRet) {
        throw std::runtime_error("Failed to get graphics/present queues");
    }
    graphicsQueue_ = graphicsQueueRet.value();
    graphicsQueueFamily_ = graphicsFamilyRet.value();
    presentQueue_ = presentQueueRet.value();

    // --- Memory allocator ----------------------------------------------------------------
    VmaAllocatorCreateInfo allocatorInfo{};
    allocatorInfo.physicalDevice = physicalDevice_.physical_device;
    allocatorInfo.device = device_.device;
    allocatorInfo.instance = instance_.instance;
    allocatorInfo.vulkanApiVersion = VK_API_VERSION_1_3;
    vkCheck(vmaCreateAllocator(&allocatorInfo, &allocator_), "vmaCreateAllocator");

    const auto& props = physicalDevice_.properties;
    log::info("Vulkan device: %s (API %u.%u.%u, driver %u)",
              props.deviceName,
              VK_API_VERSION_MAJOR(props.apiVersion),
              VK_API_VERSION_MINOR(props.apiVersion),
              VK_API_VERSION_PATCH(props.apiVersion),
              props.driverVersion);
    log::info("Validation layers: %s", useValidation ? "on" : "off");
}

VkContext::~VkContext() {
    if (allocator_ != VK_NULL_HANDLE) {
        vmaDestroyAllocator(allocator_);
    }
    vkb::destroy_device(device_);
    vkb::destroy_surface(instance_, surface_);
    vkb::destroy_instance(instance_);
}

void VkContext::waitIdle() const {
    vkCheck(vkDeviceWaitIdle(device_.device), "vkDeviceWaitIdle");
}

}  // namespace raum
