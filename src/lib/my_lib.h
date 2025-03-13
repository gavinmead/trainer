#pragma once

namespace my_lib {
    // A C++23 feature: auto non-type template parameters
    template <auto N>
    constexpr int calculate();
    
    // A function that uses C++23 features
    int get_answer();
}

