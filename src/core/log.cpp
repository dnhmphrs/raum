#include "core/log.hpp"

#include <cstdarg>
#include <cstdio>

namespace raum::log {

namespace {

void write(const char* level, const char* fmt, va_list args) {
    std::fprintf(stderr, "[%s] ", level);
    std::vfprintf(stderr, fmt, args);
    std::fputc('\n', stderr);
    std::fflush(stderr);
}

}  // namespace

void info(const char* fmt, ...) {
    va_list args;
    va_start(args, fmt);
    write("info", fmt, args);
    va_end(args);
}

void warn(const char* fmt, ...) {
    va_list args;
    va_start(args, fmt);
    write("warn", fmt, args);
    va_end(args);
}

void error(const char* fmt, ...) {
    va_list args;
    va_start(args, fmt);
    write("error", fmt, args);
    va_end(args);
}

}  // namespace raum::log
