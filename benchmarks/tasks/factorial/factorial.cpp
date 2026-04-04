// SPDX-License-Identifier: MIT-0

#include <iostream>

long long fac(int n) {
    if (n == 0) return 1;
    return n * fac(n - 1);
}

int main() {
    std::cout << fac(10) << std::endl;
    return 0;
}
