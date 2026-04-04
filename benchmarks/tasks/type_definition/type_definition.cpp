// SPDX-License-Identifier: MIT-0

#include <iostream>
#include <memory>

struct List {
    int head;
    std::shared_ptr<List> tail;
    List(int h, std::shared_ptr<List> t) : head(h), tail(t) {}
};

using ListPtr = std::shared_ptr<List>;

ListPtr nil() { return nullptr; }
ListPtr cons(int head, ListPtr tail) { return std::make_shared<List>(head, tail); }

int length(ListPtr xs) {
    if (!xs) return 0;
    return 1 + length(xs->tail);
}

ListPtr append(ListPtr xs, int x) {
    if (!xs) return cons(x, nil());
    return cons(xs->head, append(xs->tail, x));
}

int main() {
    auto xs = cons(1, cons(2, cons(3, nil())));
    auto ys = append(xs, 4);
    std::cout << length(ys) << std::endl;
    return 0;
}
