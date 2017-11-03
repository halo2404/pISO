#ifndef CONFIG_HPP
#define CONFIG_HPP

#include "error.hpp"
#include <stdlib.h>
#include <string>

constexpr char VOLUME_GROUP_NAME[] = "VolGroup00";
constexpr char THINPOOL_NAME[] = "thinpool";

inline std::string config_getenv(const char *name) {
  auto val = getenv(name);
  if (val == NULL) {
    multitool_error("getenv: cannot find '", name, "'");
  }
  return val;
}

#endif
