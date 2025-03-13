#include "lib/my_lib.h"
#include <iostream>

int main() {
    std::cout << "C++23 project with Bazel" << std::endl;
    std::cout << "The answer is: " << my_lib::get_answer() << std::endl;
    return 0;
}

