#include "my_lib.h"

#include <iostream>
#include <optional>
#include <string_view>

namespace my_lib {
    // C++23 feature example implementation
    template <>
    constexpr int calculate<42>() {
        return 42;
    }
    
    int get_answer() {
        // Using C++23's explicit objects for functions
        const auto print = []<typename... Args>(Args&&... args) {
            (std::cout << ... << args);
        };
        
        // Using if consteval (C++23)
        if consteval {
            return 43; // Compile-time context
        } else {
            return calculate<42>(); // Runtime context
        }
    }
}

