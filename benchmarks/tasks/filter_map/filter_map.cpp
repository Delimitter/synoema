// SPDX-License-Identifier: MIT-0

#include <iostream>

int main() {
    int sum = 0;
    for (int x = 1; x <= 10; ++x) {
        if (x % 2 == 0) {
            sum += x * x;
        }
    }
    std::cout << sum << std::endl;
    return 0;
}
