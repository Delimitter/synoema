# SPDX-License-Identifier: MIT-0

person = {"name": "Alice", "age": 30, "hobbies": ["reading", "coding", "hiking"]}
hobbies = " ".join(person["hobbies"])
print(f"{{name: {person['name']}, age: {person['age']}, hobbies: [{hobbies}]}}")
