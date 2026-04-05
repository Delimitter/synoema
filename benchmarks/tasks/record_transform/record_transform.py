# SPDX-License-Identifier: MIT-0

p = {"name": "Alice", "x": 0, "y": 0}
moved = {**p, "x": 3, "y": 4}
renamed = {**moved, "name": "Bob"}
print(f"{renamed['name']} at ({renamed['x']}, {renamed['y']})")
