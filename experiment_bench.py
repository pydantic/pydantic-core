import random
import string

import pyperf

from pydantic_core._pydantic_core import ModelByIter, ModelByLookup

runner = pyperf.Runner()

def foo():
    x = 0
    for i in range(100):
        if i % 2 == 0:
            x += i
    return x


fields = list(string.ascii_lowercase)

data = fields[:]
random.shuffle(data)

data = {k: i for i, k in enumerate(data)}

model_iter = ModelByIter(fields)
# model_iter.set_self_ptr()
model_lookup = ModelByLookup(fields)

iter_data = model_iter.validate(data)
lookup_data = model_lookup.validate(data)
# assert model_iter.validate(data) == model_lookup.validate(data)
assert iter_data.model_dump() == lookup_data, (iter_data.model_dict(), lookup_data)
assert iter_data.a == lookup_data['a']
assert iter_data.b == lookup_data['b']
# debug(data, iter_data.a, iter_data.b, iter_data.model_dump())

# print(data)
runner.bench_func('iter', model_iter.validate, data)
runner.bench_func('lookup', model_lookup.validate, data)
