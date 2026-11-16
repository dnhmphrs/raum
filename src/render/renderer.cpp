#include "render/renderer.hpp"

#include "core/log.hpp"
#include "core/swapchain.hpp"
#include "core/vk_check.hpp"
#include "core/vk_context.hpp"
#include "core/window.hpp"

#include <limits>

namespace raum {

namespace {

// Layout transition for a single-mip, single-layer colour image, using synchronization2.
void imageBarrier(VkCommandBuffer cmd, VkImage image,
                  VkImageLayout oldLayout, VkImageLayout newLayout,
                  VkPipelineStageFlags2 srcStage, VkAccessFlags2 srcAccess,
                  VkPipelineStageFlags2 dstStage, VkAccessFlags2 dstAccess) {
    VkImageMemoryBarrier2 barrier{};
    barrier.sType = VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER_2;
    barrier.srcStageMask = srcStage;
    barrier.srcAccessMask = srcAccess;
    barrier.dstStageMask = dstStage;
    barrier.dstAccessMask = dstAccess;
    barrier.oldLayout = oldLayout;
    barrier.newLayout = newLayout;
    barrier.srcQueueFamilyIndex = VK_QUEUE_FAMILY_IGNORED;
    barrier.dstQueueFamilyIndex = VK_QUEUE_FAMILY_IGNORED;
    barrier.image = image;
    barrier.subresourceRange.aspectMask = VK_IMAGE_ASPECT_COLOR_BIT;
    barrier.subresourceRange.baseMipLevel = 0;
    barrier.subresourceRange.levelCount = 1;
    barrier.subresourceRange.baseArrayLayer = 0;
    barrier.subresourceRange.layerCount = 1;

    VkDependencyInfo dependency{};
    dependency.sType = VK_STRUCTURE_TYPE_DEPENDENCY_INFO;
    dependency.imageMemoryBarrierCount = 1;
    dependency.pImageMemoryBarriers = &barrier;

    vkCmdPipelineBarrier2(cmd, &dependency);
}

}  // namespace

Renderer::Renderer(Window& window, VkContext& ctx, Swapchain& swapchain)
    : window_(window), ctx_(ctx), swapchain_(swapchain) {
    setClearColor(0.05f, 0.07f, 0.10f);
    createFrameData();
    createRenderFinishedSemaphores();
}

Renderer::~Renderer() {
    // The app waits for the device to go idle before destroying the renderer.
    destroyRenderFinishedSemaphores();
    destroyFrameData();
}

void Renderer::setClearColor(float r, float g, float b) {
    clearColor_.float32[0] = r;
    clearColor_.float32[1] = g;
    clearColor_.float32[2] = b;
    clearColor_.float32[3] = 1.0f;
}

void Renderer::createFrameData() {
    VkDevice device = ctx_.device();

    for (FrameData& frame : frames_) {
        VkCommandPoolCreateInfo poolInfo{};
        poolInfo.sType = VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO;
        poolInfo.flags = VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT;
        poolInfo.queueFamilyIndex = ctx_.graphicsQueueFamily();
        vkCheck(vkCreateCommandPool(device, &poolInfo, nullptr, &frame.commandPool), "vkCreateCommandPool");

        VkCommandBufferAllocateInfo allocInfo{};
        allocInfo.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO;
        allocInfo.commandPool = frame.commandPool;
        allocInfo.level = VK_COMMAND_BUFFER_LEVEL_PRIMARY;
        allocInfo.commandBufferCount = 1;
        vkCheck(vkAllocateCommandBuffers(device, &allocInfo, &frame.commandBuffer), "vkAllocateCommandBuffers");

        VkSemaphoreCreateInfo semaphoreInfo{};
        semaphoreInfo.sType = VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO;
        vkCheck(vkCreateSemaphore(device, &semaphoreInfo, nullptr, &frame.imageAvailable), "vkCreateSemaphore");

        // Created signalled so the first wait on it returns immediately.
        VkFenceCreateInfo fenceInfo{};
        fenceInfo.sType = VK_STRUCTURE_TYPE_FENCE_CREATE_INFO;
        fenceInfo.flags = VK_FENCE_CREATE_SIGNALED_BIT;
        vkCheck(vkCreateFence(device, &fenceInfo, nullptr, &frame.inFlight), "vkCreateFence");
    }
}

void Renderer::destroyFrameData() {
    VkDevice device = ctx_.device();
    for (FrameData& frame : frames_) {
        if (frame.inFlight) vkDestroyFence(device, frame.inFlight, nullptr);
        if (frame.imageAvailable) vkDestroySemaphore(device, frame.imageAvailable, nullptr);
        if (frame.commandPool) vkDestroyCommandPool(device, frame.commandPool, nullptr);  // frees the buffer too
        frame = FrameData{};
    }
}

void Renderer::createRenderFinishedSemaphores() {
    VkDevice device = ctx_.device();
    renderFinished_.resize(swapchain_.imageCount(), VK_NULL_HANDLE);
    VkSemaphoreCreateInfo semaphoreInfo{};
    semaphoreInfo.sType = VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO;
    for (VkSemaphore& semaphore : renderFinished_) {
        vkCheck(vkCreateSemaphore(device, &semaphoreInfo, nullptr, &semaphore), "vkCreateSemaphore");
    }
}

void Renderer::destroyRenderFinishedSemaphores() {
    VkDevice device = ctx_.device();
    for (VkSemaphore semaphore : renderFinished_) {
        if (semaphore) vkDestroySemaphore(device, semaphore, nullptr);
    }
    renderFinished_.clear();
}

void Renderer::recreateSwapchain() {
    // A minimised window has a 0x0 framebuffer; there is nothing to render into until
    // it comes back, so block on events rather than spin.
    VkExtent2D extent = window_.framebufferExtent();
    while ((extent.width == 0 || extent.height == 0) && !window_.shouldClose()) {
        window_.waitEvents();
        extent = window_.framebufferExtent();
    }
    if (window_.shouldClose()) {
        return;
    }

    ctx_.waitIdle();
    swapchain_.recreate(extent);
    destroyRenderFinishedSemaphores();
    createRenderFinishedSemaphores();
    window_.consumeResized();  // the rebuild covers any resize that arrived meanwhile
}

void Renderer::recordCommands(VkCommandBuffer cmd, uint32_t imageIndex) {
    VkCommandBufferBeginInfo beginInfo{};
    beginInfo.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO;
    beginInfo.flags = VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT;
    vkCheck(vkBeginCommandBuffer(cmd, &beginInfo), "vkBeginCommandBuffer");

    const VkImage image = swapchain_.image(imageIndex);

    // Swapchain image: whatever it was -> colour attachment. The acquire semaphore is
    // waited at COLOR_ATTACHMENT_OUTPUT, so that is the stage this barrier chains from.
    imageBarrier(cmd, image,
                 VK_IMAGE_LAYOUT_UNDEFINED, VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL,
                 VK_PIPELINE_STAGE_2_COLOR_ATTACHMENT_OUTPUT_BIT, VK_ACCESS_2_NONE,
                 VK_PIPELINE_STAGE_2_COLOR_ATTACHMENT_OUTPUT_BIT, VK_ACCESS_2_COLOR_ATTACHMENT_WRITE_BIT);

    VkRenderingAttachmentInfo colorAttachment{};
    colorAttachment.sType = VK_STRUCTURE_TYPE_RENDERING_ATTACHMENT_INFO;
    colorAttachment.imageView = swapchain_.imageView(imageIndex);
    colorAttachment.imageLayout = VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL;
    colorAttachment.loadOp = VK_ATTACHMENT_LOAD_OP_CLEAR;
    colorAttachment.storeOp = VK_ATTACHMENT_STORE_OP_STORE;
    colorAttachment.clearValue.color = clearColor_;

    VkRenderingInfo renderingInfo{};
    renderingInfo.sType = VK_STRUCTURE_TYPE_RENDERING_INFO;
    renderingInfo.renderArea.offset = {0, 0};
    renderingInfo.renderArea.extent = swapchain_.extent();
    renderingInfo.layerCount = 1;
    renderingInfo.colorAttachmentCount = 1;
    renderingInfo.pColorAttachments = &colorAttachment;

    vkCmdBeginRendering(cmd, &renderingInfo);
    // Milestone 1 binds a pipeline and draws here.
    vkCmdEndRendering(cmd);

    // Colour attachment -> presentable.
    imageBarrier(cmd, image,
                 VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL, VK_IMAGE_LAYOUT_PRESENT_SRC_KHR,
                 VK_PIPELINE_STAGE_2_COLOR_ATTACHMENT_OUTPUT_BIT, VK_ACCESS_2_COLOR_ATTACHMENT_WRITE_BIT,
                 VK_PIPELINE_STAGE_2_BOTTOM_OF_PIPE_BIT, VK_ACCESS_2_NONE);

    vkCheck(vkEndCommandBuffer(cmd), "vkEndCommandBuffer");
}

void Renderer::drawFrame() {
    VkDevice device = ctx_.device();
    FrameData& frame = frames_[frameIndex_];
    constexpr uint64_t kNoTimeout = std::numeric_limits<uint64_t>::max();

    // Wait until the GPU has finished with this frame slot's command buffer.
    vkCheck(vkWaitForFences(device, 1, &frame.inFlight, VK_TRUE, kNoTimeout), "vkWaitForFences");

    uint32_t imageIndex = 0;
    const VkResult acquireResult = vkAcquireNextImageKHR(device, swapchain_.handle(), kNoTimeout,
                                                         frame.imageAvailable, VK_NULL_HANDLE, &imageIndex);
    if (acquireResult == VK_ERROR_OUT_OF_DATE_KHR) {
        recreateSwapchain();
        return;  // fence is still signalled; next call will wait on it without blocking
    }
    if (acquireResult != VK_SUCCESS && acquireResult != VK_SUBOPTIMAL_KHR) {
        vkCheck(acquireResult, "vkAcquireNextImageKHR");
    }

    // Only reset the fence once we know we will submit work that signals it.
    vkCheck(vkResetFences(device, 1, &frame.inFlight), "vkResetFences");
    vkCheck(vkResetCommandBuffer(frame.commandBuffer, 0), "vkResetCommandBuffer");

    recordCommands(frame.commandBuffer, imageIndex);

    VkSemaphoreSubmitInfo waitInfo{};
    waitInfo.sType = VK_STRUCTURE_TYPE_SEMAPHORE_SUBMIT_INFO;
    waitInfo.semaphore = frame.imageAvailable;
    waitInfo.stageMask = VK_PIPELINE_STAGE_2_COLOR_ATTACHMENT_OUTPUT_BIT;

    VkSemaphoreSubmitInfo signalInfo{};
    signalInfo.sType = VK_STRUCTURE_TYPE_SEMAPHORE_SUBMIT_INFO;
    signalInfo.semaphore = renderFinished_[imageIndex];
    signalInfo.stageMask = VK_PIPELINE_STAGE_2_ALL_GRAPHICS_BIT;

    VkCommandBufferSubmitInfo commandInfo{};
    commandInfo.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_SUBMIT_INFO;
    commandInfo.commandBuffer = frame.commandBuffer;

    VkSubmitInfo2 submitInfo{};
    submitInfo.sType = VK_STRUCTURE_TYPE_SUBMIT_INFO_2;
    submitInfo.waitSemaphoreInfoCount = 1;
    submitInfo.pWaitSemaphoreInfos = &waitInfo;
    submitInfo.commandBufferInfoCount = 1;
    submitInfo.pCommandBufferInfos = &commandInfo;
    submitInfo.signalSemaphoreInfoCount = 1;
    submitInfo.pSignalSemaphoreInfos = &signalInfo;
    vkCheck(vkQueueSubmit2(ctx_.graphicsQueue(), 1, &submitInfo, frame.inFlight), "vkQueueSubmit2");

    VkSwapchainKHR swapchainHandle = swapchain_.handle();
    VkPresentInfoKHR presentInfo{};
    presentInfo.sType = VK_STRUCTURE_TYPE_PRESENT_INFO_KHR;
    presentInfo.waitSemaphoreCount = 1;
    presentInfo.pWaitSemaphores = &renderFinished_[imageIndex];
    presentInfo.swapchainCount = 1;
    presentInfo.pSwapchains = &swapchainHandle;
    presentInfo.pImageIndices = &imageIndex;
    const VkResult presentResult = vkQueuePresentKHR(ctx_.presentQueue(), &presentInfo);

    const bool resized = window_.consumeResized();
    if (presentResult == VK_ERROR_OUT_OF_DATE_KHR || presentResult == VK_SUBOPTIMAL_KHR || resized) {
        recreateSwapchain();
    } else {
        vkCheck(presentResult, "vkQueuePresentKHR");
    }

    frameIndex_ = (frameIndex_ + 1) % kFramesInFlight;
}

}  // namespace raum
