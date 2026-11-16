#pragma once

#include <vulkan/vulkan.h>
// GLFW declares its Vulkan helpers only when vulkan.h was included first.
#include <GLFW/glfw3.h>

#include <string>

namespace raum {

// Owns the GLFW library lifetime and a single window. No rendering API is attached;
// Vulkan talks to it through the surface created in createSurface().
class Window {
public:
    Window(int width, int height, const std::string& title);
    ~Window();

    Window(const Window&) = delete;
    Window& operator=(const Window&) = delete;

    bool shouldClose() const;
    void pollEvents() const;
    void waitEvents() const;  // blocks until an event arrives (used while minimised)

    VkExtent2D framebufferExtent() const;
    void setTitle(const std::string& title) const;

    // True once per resize event; the renderer polls this to rebuild the swapchain.
    bool consumeResized();

    VkSurfaceKHR createSurface(VkInstance instance) const;

    GLFWwindow* handle() const { return window_; }

private:
    static void framebufferSizeCallback(GLFWwindow* window, int width, int height);
    static void keyCallback(GLFWwindow* window, int key, int scancode, int action, int mods);

    GLFWwindow* window_ = nullptr;
    bool resized_ = false;
};

}  // namespace raum
