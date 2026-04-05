# SPDX-License-Identifier: MIT-0

def validate_name(name):
    if name == "":
        return ("err", "empty name")
    return ("ok", name)

def validate_age(age):
    if age < 0:
        return ("err", "negative age")
    return ("ok", age)

def and_then(result, f):
    if result[0] == "err":
        return result
    return f(result[1])

def validate(name, age):
    return and_then(validate_name(name),
        lambda n: and_then(validate_age(age),
            lambda a: ("ok", f"{n} is {a}")))

def show_result(r):
    if r[0] == "ok":
        return f"Ok {r[1]}"
    return f"Err {r[1]}"

print(show_result(validate("Alice", 25)))
print(show_result(validate("", -1)))
