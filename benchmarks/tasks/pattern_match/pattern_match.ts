// SPDX-License-Identifier: MIT-0

type Shape =
    | { type: "Circle"; r: number }
    | { type: "Rectangle"; w: number; h: number }
    | { type: "Triangle"; b: number; h: number };

function area(shape: Shape): number {
    switch (shape.type) {
        case "Circle": return 3.14 * shape.r * shape.r;
        case "Rectangle": return shape.w * shape.h;
        case "Triangle": return 0.5 * shape.b * shape.h;
    }
}

function fmt(n: number): string { return n === Math.floor(n) ? String(Math.floor(n)) : String(n); }

const shapes: { name: string; shape: Shape }[] = [
    { name: "Circle", shape: { type: "Circle", r: 5.0 } },
    { name: "Rectangle", shape: { type: "Rectangle", w: 4.0, h: 6.0 } },
    { name: "Triangle", shape: { type: "Triangle", b: 6.0, h: 5.0 } },
];

for (const { name, shape } of shapes) {
    console.log(`${name}: ${fmt(area(shape))}`);
}
