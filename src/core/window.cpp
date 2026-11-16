#include "core/window.hpp"

#include "core/log.hpp"

#include <stdexcept>

namespace raum {

namespace {

void glfwErrorCallback(int code, const char* description) {
    log::error("GLFW error %d: %s", code, description);
}

}  // namespace

Window::Window(int width, int height, const std::string& title) {
    glfwSetErrorCallback(glfwErrorCallback);
    if (!glfwInit()) {
        throw std::runtime_error("glfwInit failed");
    }
    if (!glfwVulkanSupported()) {
        glfwTerminate();
        throw std::runtime_error("GLFW could not find a Vulkan loader or ICD. Is the Vulkan SDK / driver installed?");
    }

    glfwWindowHint(GLFW_CLIENT_API, GLFW_NO_API);  // no OpenGL context, Vulkan only
    window_ = glfwCreateWindow(width, height, title.c_str(), nullptr, nullptr);
    if (!window_) {
        glfwTerminate();
        throw std::runtime_error("glfwCreateWindow failed");
    }

    glfwSetWindowUserPointer(window_, this);
    glfwSetFramebufferSizeCallback(window_, framebufferSizeCallback);
    glfwSetKeyCallback(window_, keyCallback);
}

Window::~Window() {
    if (window_) {
        glfwDestroyWindow(window_);
    }
    glfwTerminate();
}

bool Window::shouldClose() const {
    return glfwWindowShouldClose(window_) != 0;
}

void Window::pollEvents() const {
    glfwPollEvents();
}

void Window::waitEvents() const {
    glfwWaitEvents();
}

VkExtent2D Window::framebufferExtent() const {
    int width = 0;
    int height = 0;
    glfwGetFramebufferSize(window_, &width, &height);
    return VkExtent2D{static_cast<uint32_t>(width), static_cast<uint32_t>(height)};
}

void Window::setTitle(const std::string& title) const {
    glfwSetWindowTitle(window_, title.c_str());
}

bool Window::consumeResized() {
    const bool was = resized_;
    resized_ = false;
    return was;
}

VkSurfaceKHR Window::createSurface(VkInstance instance) const {
    VkSurfaceKHR surface = VK_NULL_HANDLE;
    const VkResult result = glfwCreateWindowSurface(instance, window_, nullptr, &surface);
    if (result != VK_SUCCESS) {
        throw std::runtime_error("glfwCreateWindowSurface failed");
    }
    return surface;
}

void Window::framebufferSizeCallback(GLFWwindow* window, int /*width*/, int /*height*/) {
    auto* self = static_cast<Window*>(glfwGetWindowUserPointer(window));
    if (self) {
        self->resized_ = true;
    }
}

void Window::keyCallback(GLFWwindow* window, int key, int /*scancode*/, int action, int /*mods*/) {
    if (key == GLFW_KEY_ESCAPE && action == GLFW_PRESS) {
        glfwSetWindowShouldClose(window, GLFW_TRUE);
    }
}

}  // namespace raum
