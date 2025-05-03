#pragma once
#include <string>
#include <unordered_map>

class EnvArgs {
public:
    EnvArgs(int argc, char** argv);
    size_t get_usize(const std::string& name, size_t fallback = 1) const;
    bool get_bool(const std::string& name, bool fallback = false) const;
private:
    std::unordered_map<std::string, std::string> map;
};