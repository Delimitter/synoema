// SPDX-License-Identifier: MIT-0

#include <iostream>
#include <string>

std::string fizzbuzz(int n) {
    if (n % 15 == 0) return "FizzBuzz";
    if (n % 3 == 0) return "Fizz";
    if (n % 5 == 0) return "Buzz";
    return std::to_string(n);
}

int main() {
    std::cout << fizzbuzz(15) << std::endl;
    return 0;
}
