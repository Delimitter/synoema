// SPDX-License-Identifier: MIT-0

type Tree = { type: "leaf"; value: number } | { type: "node"; left: Tree; value: number; right: Tree };

function leaf(value: number): Tree { return { type: "leaf", value }; }
function node(left: Tree, value: number, right: Tree): Tree { return { type: "node", left, value, right }; }

function inorder(tree: Tree): number[] {
    if (tree.type === "leaf") return [tree.value];
    return [...inorder(tree.left), tree.value, ...inorder(tree.right)];
}

const tree = node(node(leaf(1), 3, leaf(4)), 5, node(leaf(6), 7, leaf(9)));
console.log(inorder(tree).join(" "));
