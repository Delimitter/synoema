// SPDX-License-Identifier: MIT-0

#include <iostream>
#include <vector>

int binarySearch(const std::vector<int>& xs, int target) {
    int lo = 0, hi = static_cast<int>(xs.size()) - 1;
    while (lo <= hi) {
        int mid = (lo + hi) / 2;
        if (xs[mid] == target) return mid;
        else if (xs[mid] < target) lo = mid + 1;
        else hi = mid - 1;
    }
    return -1;
}

int main() {
    std::vector<int> xs = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10};
    std::cout << binarySearch(xs, 7) << std::endl;
    return 0;
}
