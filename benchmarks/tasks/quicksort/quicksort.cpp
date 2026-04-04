// SPDX-License-Identifier: MIT-0

#include <iostream>
#include <vector>

std::vector<int> qsort(std::vector<int> lst) {
    if (lst.empty()) return {};
    int p = lst[0];
    std::vector<int> lo, hi;
    for (size_t i = 1; i < lst.size(); ++i) {
        if (lst[i] <= p) lo.push_back(lst[i]);
        else hi.push_back(lst[i]);
    }
    auto sorted_lo = qsort(lo);
    auto sorted_hi = qsort(hi);
    sorted_lo.push_back(p);
    sorted_lo.insert(sorted_lo.end(), sorted_hi.begin(), sorted_hi.end());
    return sorted_lo;
}

int main() {
    auto result = qsort({5, 3, 8, 1, 9, 2, 7, 4, 6});
    std::cout << "[";
    for (size_t i = 0; i < result.size(); ++i) {
        if (i > 0) std::cout << " ";
        std::cout << result[i];
    }
    std::cout << "]" << std::endl;
    return 0;
}
