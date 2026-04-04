// SPDX-License-Identifier: MIT-0

#include <iostream>
#include <vector>
#include <variant>
#include <memory>

struct Leaf { int value; };
struct Node;
using Tree = std::variant<Leaf, std::unique_ptr<Node>>;
struct Node {
    Tree left;
    int value;
    Tree right;
    Node(Tree l, int v, Tree r) : left(std::move(l)), value(v), right(std::move(r)) {}
};

void inorder(const Tree& tree, std::vector<int>& result) {
    if (auto* l = std::get_if<Leaf>(&tree)) {
        result.push_back(l->value);
    } else {
        auto& n = *std::get<std::unique_ptr<Node>>(tree);
        inorder(n.left, result);
        result.push_back(n.value);
        inorder(n.right, result);
    }
}

Tree makeLeaf(int v) { return Leaf{v}; }
Tree makeNode(Tree l, int v, Tree r) {
    return std::make_unique<Node>(std::move(l), v, std::move(r));
}

int main() {
    auto tree = makeNode(
        makeNode(makeLeaf(1), 3, makeLeaf(4)),
        5,
        makeNode(makeLeaf(6), 7, makeLeaf(9))
    );
    std::vector<int> result;
    inorder(tree, result);
    for (size_t i = 0; i < result.size(); ++i) {
        if (i > 0) std::cout << " ";
        std::cout << result[i];
    }
    std::cout << std::endl;
    return 0;
}
