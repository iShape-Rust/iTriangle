#include "args.h"
#include <cstring>
#include <cstdlib>

EnvArgs::EnvArgs(int argc, char** argv) {
    for (int i = 1; i < argc; ++i) {
        if (strncmp(argv[i], "--", 2) == 0) {
            std::string key = argv[i] + 2;
            std::string value = (i + 1 < argc && strncmp(argv[i + 1], "--", 2) != 0) ? argv[++i] : "true";
            map[key] = value;
        }
    }
}

size_t EnvArgs::get_usize(const std::string& name, size_t fallback) const {
    auto it = map.find(name);
    return it != map.end() ? std::stoul(it->second) : fallback;
}

bool EnvArgs::get_bool(const std::string& name, bool fallback) const {
    auto it = map.find(name);
    if (it == map.end()) return fallback;
    return it->second == "true" || it->second == "1";
}
