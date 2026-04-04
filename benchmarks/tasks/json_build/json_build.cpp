// SPDX-License-Identifier: MIT-0

#include <iostream>
#include <string>
#include <vector>

struct Person {
    std::string name;
    int age;
    std::vector<std::string> hobbies;
};

int main() {
    Person person = {"Alice", 30, {"reading", "coding", "hiking"}};
    std::cout << "{name: " << person.name
              << ", age: " << person.age
              << ", hobbies: [";
    for (size_t i = 0; i < person.hobbies.size(); ++i) {
        if (i > 0) std::cout << " ";
        std::cout << person.hobbies[i];
    }
    std::cout << "]}" << std::endl;
    return 0;
}
