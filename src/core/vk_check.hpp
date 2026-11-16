#pragma once

#include <vulkan/vulkan.h>

#include <stdexcept>
#include <string>

namespace raum {

// Throws if a Vulkan call did not return VK_SUCCESS. `what` names the call for the message.
inline void vkCheck(VkResult result, const char* what) {
    if (result != VK_SUCCESS) {
        throw std::runtime_error(std::string(what) + " failed (VkResult " +
                                 std::to_string(static_cast<int>(result)) + ")");
    }
}

}  // namespace raum
