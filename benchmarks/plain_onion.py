from functools import partial


class Onion:
    def __init__(self, funcs):
        func, *layers = reversed(funcs)
        for layer in layers:
            func = partial(layer, handler=func)
        self.func = func

    def __call__(self, arg):
        return self.func(arg)


def to_str(value):
    return str(value)


def max_length(value, handler):
    s = handler(value).upper()
    if len(s) > 10:
        raise ValueError('Value is too long')
    return s


def strip_whitespace(value, handler):
    s = handler(value)
    return s.strip()


def prepend(value, handler):
    return handler('x' + value)


functions = [prepend, strip_whitespace, max_length, to_str]
