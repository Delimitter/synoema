// SPDX-License-Identifier: MIT-0

function leaf(value) { return { type: "leaf", value }; }
function node(left, value, right) { return { type: "node", left, value, right }; }

function inorder(tree) {
    if (tree.type === "leaf") return [tree.value];
    return [...inorder(tree.left), tree.value, ...inorder(tree.right)];
}

const tree = node(node(leaf(1), 3, leaf(4)), 5, node(leaf(6), 7, leaf(9)));
console.log(inorder(tree).join(" "));
