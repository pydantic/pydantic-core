#!/usr/bin/env python3
"""
This is used to generate flamegraphs, usage:

    perf record -g benchmarks/minimal.py

Then:

    perf script | stackcollapse-perf.pl | flamegraph.pl > flame.svg

As per https://gist.github.com/KodrAus/97c92c07a90b1fdd6853654357fd557a
"""
from pydantic_core import SchemaValidator, ValidationError
import json

v = SchemaValidator({
    'title': 'MyTestModel',
    'type': 'model',
    'fields': {
        'name': {
            'type': 'str',
        },
        'age': {
            'type': 'int-constrained',
            'ge': 18,
        },
        'is_employer': {
            'type': 'bool',
            'default': True,
        },
        'friends': {
            'type': 'list',
            'items': {
                'type': 'int',
                'gt': 0,
            },
        },
        'settings': {
            'type': 'dict',
            'keys': {
                'type': 'str',
            },
            'values': {
                'type': 'float',
            }
        }
    },
})
# print(repr(v))
d = {'name': 'John', 'age': 42, 'friends': list(range(200000)), 'settings': {f'v_{i}': i / 2.0 for i in range(500)}}
# r = v.validate_python(d)
r = v.validate_json(json.dumps(d))
# print(r)

try:
    r = v.validate_python({'name': 'John', 'age': 16, 'friends': [-1, 2, 3, -1], 'settings': {'a': 1.0, 'b': 2.0}})
except ValidationError as e:
    # print(e)
    pass
