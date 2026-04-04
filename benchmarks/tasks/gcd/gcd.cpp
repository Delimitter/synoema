// SPDX-License-Identifier: MIT-0

#include <iostream>

int gcd(int a, int b) {
    if (b == 0) return a;
    return gcd(b, a % b);
}

int main() {
    std::cout << gcd(48, 18) << std::endl;
    return 0;
}
