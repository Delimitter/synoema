// SPDX-License-Identifier: MIT-0

#include <iostream>
#include <string>
#include <variant>
#include <cmath>

struct Circle { double r; };
struct Rectangle { double w; double h; };
struct Triangle { double b; double h; };
using Shape = std::variant<Circle, Rectangle, Triangle>;

double area(const Shape& shape) {
    if (auto* c = std::get_if<Circle>(&shape))
        return 3.14 * c->r * c->r;
    if (auto* r = std::get_if<Rectangle>(&shape))
        return r->w * r->h;
    auto& t = std::get<Triangle>(shape);
    return 0.5 * t.b * t.h;
}

void printArea(const std::string& name, const Shape& shape) {
    double a = area(shape);
    std::cout << name << ": ";
    if (a == std::floor(a))
        std::cout << static_cast<int>(a);
    else
        std::cout << a;
    std::cout << std::endl;
}

int main() {
    printArea("Circle", Circle{5.0});
    printArea("Rectangle", Rectangle{4.0, 6.0});
    printArea("Triangle", Triangle{6.0, 5.0});
    return 0;
}
