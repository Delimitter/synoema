// SPDX-License-Identifier: MIT-0

#include <iostream>
#include <string>
#include <variant>

struct Ok { int value; };
struct Err { std::string message; };
using Result = std::variant<Ok, Err>;

Result divide(int x, int y) {
    if (y == 0) return Err{"division by zero"};
    return Ok{x / y};
}

std::string showResult(const Result& r) {
    if (auto* e = std::get_if<Err>(&r))
        return "Err " + e->message;
    return "Ok " + std::to_string(std::get<Ok>(r).value);
}

int main() {
    std::cout << showResult(divide(10, 0)) << std::endl;
    std::cout << showResult(divide(10, 2)) << std::endl;
    return 0;
}
