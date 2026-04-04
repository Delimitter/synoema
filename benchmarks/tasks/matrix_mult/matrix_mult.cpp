// SPDX-License-Identifier: MIT-0

#include <iostream>
#include <vector>

using Matrix = std::vector<std::vector<int>>;

Matrix matMult(const Matrix& a, const Matrix& b) {
    int n = static_cast<int>(a.size());
    Matrix result(n, std::vector<int>(n, 0));
    for (int i = 0; i < n; i++)
        for (int j = 0; j < n; j++)
            for (int k = 0; k < n; k++)
                result[i][j] += a[i][k] * b[k][j];
    return result;
}

int main() {
    Matrix a = {{1, 2, 3}, {4, 5, 6}, {7, 8, 9}};
    Matrix b = {{1, 2, 3}, {4, 5, 6}, {7, 8, 9}};
    Matrix result = matMult(a, b);
    bool first = true;
    for (const auto& row : result)
        for (int x : row) {
            if (!first) std::cout << " ";
            std::cout << x;
            first = false;
        }
    std::cout << std::endl;
    return 0;
}
