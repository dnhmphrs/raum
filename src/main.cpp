#include "app.hpp"
#include "core/log.hpp"

#include <cstdio>
#include <cstdlib>
#include <exception>
#include <string_view>

namespace {

void printUsage() {
    std::printf(
        "raum [options]\n"
        "  --frames N        render N frames then exit (smoke test)\n"
        "  --validation      enable Vulkan validation layers (default in Debug builds)\n"
        "  --no-validation   disable validation layers\n"
        "  -h, --help        show this help\n"
        "Esc closes the window.\n");
}

}  // namespace

int main(int argc, char** argv) {
    raum::AppConfig config;
#ifdef NDEBUG
    config.validation = false;
#else
    config.validation = true;
#endif

    for (int i = 1; i < argc; ++i) {
        const std::string_view arg = argv[i];
        if (arg == "--frames" && i + 1 < argc) {
            config.maxFrames = std::atoi(argv[++i]);
        } else if (arg == "--validation") {
            config.validation = true;
        } else if (arg == "--no-validation") {
            config.validation = false;
        } else if (arg == "-h" || arg == "--help") {
            printUsage();
            return 0;
        } else {
            raum::log::error("Unknown argument: %s", argv[i]);
            printUsage();
            return 2;
        }
    }

    try {
        raum::App app(config);
        app.run();
    } catch (const std::exception& e) {
        raum::log::error("Fatal: %s", e.what());
        return 1;
    }
    return 0;
}
