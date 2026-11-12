#pragma once

// Minimal logging: printf-style, to stderr. Enough for now; swap for something
// richer (spdlog, in-app console) when the UI milestone needs it.

#if defined(__GNUC__) || defined(__clang__)
#define RAUM_PRINTF_LIKE(fmtIndex, firstArg) __attribute__((format(printf, fmtIndex, firstArg)))
#else
#define RAUM_PRINTF_LIKE(fmtIndex, firstArg)
#endif

namespace raum::log {

void info(const char* fmt, ...) RAUM_PRINTF_LIKE(1, 2);
void warn(const char* fmt, ...) RAUM_PRINTF_LIKE(1, 2);
void error(const char* fmt, ...) RAUM_PRINTF_LIKE(1, 2);

}  // namespace raum::log
