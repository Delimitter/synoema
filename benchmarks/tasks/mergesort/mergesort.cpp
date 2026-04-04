// SPDX-License-Identifier: MIT-0

#include <iostream>
#include <vector>

std::vector<int> merge(const std::vector<int>& xs, const std::vector<int>& ys) {
    std::vector<int> result;
    size_t i = 0, j = 0;
    while (i < xs.size() && j < ys.size()) {
        if (xs[i] <= ys[j]) result.push_back(xs[i++]);
        else result.push_back(ys[j++]);
    }
    while (i < xs.size()) result.push_back(xs[i++]);
    while (j < ys.size()) result.push_back(ys[j++]);
    return result;
}

std::vector<int> msort(std::vector<int> lst) {
    if (lst.size() <= 1) return lst;
    size_t n = lst.size() / 2;
    std::vector<int> left(lst.begin(), lst.begin() + n);
    std::vector<int> right(lst.begin() + n, lst.end());
    return merge(msort(left), msort(right));
}

int main() {
    auto result = msort({5, 3, 8, 1, 9, 2, 7, 4, 6});
    std::cout << "[";
    for (size_t i = 0; i < result.size(); ++i) {
        if (i > 0) std::cout << " ";
        std::cout << result[i];
    }
    std::cout << "]" << std::endl;
    return 0;
}
