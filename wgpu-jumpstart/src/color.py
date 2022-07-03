import sys
import math


def into_linear(r, g, b):
    def linear_component(u):
        if u < 0.04045:
            return u / 12.92
        else:
            return math.pow((u + 0.055) / 1.055, 2.4)

    return [
        linear_component(r/255.0),
        linear_component(g/255.0),
        linear_component(b/255.0),
    ]


args = sys.argv[1:]
lin = into_linear(float(args[0]), float(args[1]), float(args[2]))

print(lin)
