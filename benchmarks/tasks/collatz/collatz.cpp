// SPDX-License-Identifier: MIT-0

#include <iostream>

int collatz(long long n) {
    if (n == 1) return 0;
    if (n % 2 == 0) return 1 + collatz(n / 2);
    return 1 + collatz(3 * n + 1);
}

int main() {
    std::cout << collatz(27) << std::endl;
    return 0;
}
