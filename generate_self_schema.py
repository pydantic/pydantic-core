"""
This script generates the schema for the schema - e.g.
a definition of what inputs can be provided to `SchemaValidator()`.

The schema is generated from `pydantic_core/_types.py`.
"""
import importlib.util
import re
from collections.abc import Callable
from datetime import date, datetime, time, timedelta
from pathlib import Path
from typing import Any, Dict, ForwardRef, List, Type, Union

from black import Mode, TargetVersion, format_file_contents
from typing_extensions import get_args, is_typeddict

try:
    from typing import get_origin
except ImportError:

    def get_origin(t):
        return getattr(t, '__origin__', None)


import sys

sys
sys.setrecursionlimit(100)
THIS_DIR = Path(__file__).parent
SAVE_PATH = THIS_DIR / 'src' / 'self_schema.py'

# can't import _types.py directly as pydantic-core might not be installed
core_types_spec = importlib.util.spec_from_file_location('_typing', str(THIS_DIR / 'pydantic_core' / '_types.py'))
core_types = importlib.util.module_from_spec(core_types_spec)
core_types_spec.loader.exec_module(core_types)

# the validator for referencing schema (Schema is used recursively, so has to use a reference)
schema_ref_validator = {'type': 'recursive-ref', 'schema_ref': 'root-schema'}


def get_schema(obj):
    if isinstance(obj, str):
        return obj
    elif obj in (datetime, timedelta, date, time, bool, int, float, str):
        return obj.__name__
    elif is_typeddict(obj):
        return type_dict_schema(obj)
    elif obj == Any:
        return 'any'
    elif obj == type:
        # todo
        return 'any'

    origin = get_origin(obj)
    assert origin is not None, f'origin cannot be None, obj={obj}'
    if origin is Union:
        return union_schema(obj)
    elif obj is Callable or origin is Callable:
        return 'callable'
    elif origin is core_types.Literal:
        expected = all_literal_values(obj)
        assert expected, f'literal "expected" cannot be empty, obj={obj}'
        return {'type': 'literal', 'expected': expected}
    elif issubclass(origin, List):
        return {'type': 'list', 'items_schema': get_schema(obj.__args__[0])}
    elif issubclass(origin, Dict):
        return {
            'type': 'dict',
            'keys_schema': get_schema(obj.__args__[0]),
            'values_schema': get_schema(obj.__args__[1]),
        }
    elif issubclass(origin, Type):
        # can't really use 'is-instance' since this is used for the class_ parameter of
        # 'is-instance' validators
        return 'any'
    else:
        # debug(obj)
        raise TypeError(f'Unknown type: {obj!r}')


def type_dict_schema(typed_dict):
    required_keys = typed_dict.__required_keys__
    fields = {}
    if typed_dict.__name__ == 'Schema':
        return {'type': 'typed-dict', 'fields': fields}

    for field_name, field_type in typed_dict.__annotations__.items():
        required = field_name in required_keys
        schema = None
        if type(field_type) == ForwardRef:
            fr_arg = field_type.__forward_arg__
            fr_arg, matched = re.subn(r'NotRequired\[(.+)]', r'\1', fr_arg)
            if matched:
                required = False

            fr_arg, matched = re.subn(r'Required\[(.+)]', r'\1', fr_arg)
            if matched:
                required = True

            if 'Schema' == fr_arg or re.search('[^a-zA-Z]Schema', fr_arg):
                if fr_arg == 'Schema':
                    schema = schema_ref_validator
                elif fr_arg == 'List[Schema]':
                    schema = {'type': 'list', 'items_schema': schema_ref_validator}
                elif fr_arg == 'Dict[str, Schema]':
                    schema = {'type': 'dict', 'keys_schema': 'str', 'values_schema': schema_ref_validator}
                else:
                    raise ValueError(f'Unknown Schema forward ref: {fr_arg}')
            else:
                field_type = eval_forward_ref(field_type)

        if schema is None:
            if get_origin(field_type) == core_types.Required:
                required = True
                field_type = field_type.__args__[0]
            if get_origin(field_type) == core_types.NotRequired:
                required = False
                field_type = field_type.__args__[0]

            schema = get_schema(field_type)

        fields[field_name] = {'schema': schema, 'required': required}

    return {'type': 'typed-dict', 'description': typed_dict.__name__, 'fields': fields}


def union_schema(union_type):
    return {'type': 'union', 'choices': [get_schema(arg) for arg in union_type.__args__]}


def all_literal_values(type_):
    if get_origin(type_) is core_types.Literal:
        values = get_args(type_)
        return [x for value in values for x in all_literal_values(value)]
    else:
        return [type_]


def eval_forward_ref(type_):
    try:
        return type_._evaluate(core_types.__dict__, None, set())
    except TypeError:
        # for older python (3.7 at least)
        return type_._evaluate(core_types.__dict__, None)


def root_union_choices(schema_union):
    choices = {'function': {'type': 'union', 'choices': []}}
    for s in schema_union.__args__[1:]:
        type_ = s.__annotations__['type']
        m = re.search(r"Literal\['(.+?)']", type_.__forward_arg__)
        assert m, f'Unknown schema type: {type_}'
        key = m.group(1)
        if key == 'function':
            choices[key]['choices'].append(get_schema(s))
        else:
            choices[key] = get_schema(s)
    return choices


def main():
    schema_union = core_types.Schema
    assert get_origin(schema_union) is Union, 'expected pydantic_core._types.Schema to be a union'

    schema = {
        'type': 'union',
        'ref': 'root-schema',
        'choices': [
            get_schema(schema_union.__args__[0]),
            {'type': 'tagged-union', 'tag_key': 'type', 'choices': root_union_choices(schema_union)},
        ],
    }

    python_code = (
        f'# this file is auto-generated by generate_self_schema.py, DO NOT edit manually\nself_schema = {schema}\n'
    )
    mode = Mode(
        line_length=120,
        string_normalization=False,
        magic_trailing_comma=False,
        target_versions={TargetVersion.PY37, TargetVersion.PY38, TargetVersion.PY39, TargetVersion.PY310},
    )
    python_code = format_file_contents(python_code, fast=False, mode=mode)
    SAVE_PATH.write_text(python_code)
    print(f'Self schema definition written to {SAVE_PATH}')


if __name__ == '__main__':
    main()
