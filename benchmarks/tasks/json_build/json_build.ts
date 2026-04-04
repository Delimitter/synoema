// SPDX-License-Identifier: MIT-0

interface Person {
    name: string;
    age: number;
    hobbies: string[];
}

const person: Person = { name: "Alice", age: 30, hobbies: ["reading", "coding", "hiking"] };
const hobbies = person.hobbies.join(" ");
console.log(`{name: ${person.name}, age: ${person.age}, hobbies: [${hobbies}]}`);
