def complete_schema():
    class MyModel:
        # __slots__ is not required, but it avoids __fields_set__ falling into __dict__
        __slots__ = '__dict__', '__fields_set__'

    def append_func(input_value, **kwargs):
        return f'{input_value} Changed'

    def wrap_function(input_value, *, validator, **kwargs):
        return f'Input {validator(input_value)} Changed'

    return {
        'type': 'model-class',
        'class_type': MyModel,
        'model': {
            'type': 'model',
            'return_fields_set': True,
            'fields': {
                'field_str': {'schema': 'str'},
                'field_str_con': {'schema': {'type': 'str', 'min_length': 3, 'max_length': 5, 'pattern': '^[a-z]+$'}},
                'field_int': {'schema': 'int'},
                'field_int_con': {'schema': {'type': 'int', 'gt': 1, 'lt': 10, 'multiple_of': 2}},
                'field_float': {'schema': 'float'},
                'field_float_con': {'schema': {'type': 'float', 'ge': 1.0, 'le': 10.0, 'multiple_of': 0.5}},
                'field_bool': {'schema': 'bool'},
                'field_bytes': {'schema': 'bytes'},
                'field_bytes_con': {'schema': {'type': 'bytes', 'min_length': 6, 'max_length': 1000}},
                'field_date': {'schema': 'date'},
                'field_date_con': {'schema': {'type': 'date', 'ge': '2020-01-01', 'lt': '2020-01-02'}},
                'field_time': {'schema': 'time'},
                'field_time_con': {'schema': {'type': 'time', 'ge': '06:00:00', 'lt': '12:13:14'}},
                'field_datetime': {'schema': 'datetime'},
                'field_datetime_con': {'schema': {'type': 'datetime', 'ge': '2000-01-01T06:00:00', 'lt': '2020-01-02T12:13:14'}},
                'field_list_any': {'schema': 'list'},
                'field_list_str': {'schema': {'type': 'list', 'items': 'str'}},
                'field_list_str_con': {'schema': {'type': 'list', 'items': 'str', 'min_items': 3, 'max_items': 42}},
                'field_set_any': {'schema': 'set'},
                'field_set_int': {'schema': {'type': 'set', 'items': 'int'}},
                'field_set_int_con': {'schema': {'type': 'set', 'items': 'int', 'min_items': 3, 'max_items': 42}},
                'field_frozenset_any': {'schema': 'frozenset'},
                'field_frozenset_bytes': {'schema': {'type': 'frozenset', 'items': 'bytes'}},
                'field_frozenset_bytes_con': {'schema': {'type': 'frozenset', 'items': 'bytes', 'min_items': 3, 'max_items': 42}},
                'field_tuple_var_len_any': {'schema': 'tuple-var-len'},
                'field_tuple_var_len_float': {'schema': {'type': 'tuple-var-len', 'items': 'float'}},
                'field_tuple_var_len_float_con': {'schema': {'type': 'tuple-var-len', 'items': 'float', 'min_items': 3, 'max_items': 42}},
                'field_tuple_fix_len': {'schema': {'type': 'tuple-fix-len', 'items': ['str', 'int', 'float', 'bool']}},
                'field_dict_any': {'schema': 'dict'},
                'field_dict_str_float': {'schema': {'type': 'dict', 'key': 'str', 'values': 'float'}},
                'field_literal_1_int': {'schema': {'type': 'literal', 'expected': [1]}},
                'field_literal_1_str': {'schema': {'type': 'literal', 'expected': ['foobar']}},
                'field_literal_mult_int': {'schema': {'type': 'literal', 'expected': [1, 2, 3]}},
                'field_literal_mult_str': {'schema': {'type': 'literal', 'expected': ['foo', 'bar', 'baz']}},
                'field_literal_assorted': {'schema': {'type': 'literal', 'expected': [1, 'foo', True]}},
                'field_list_nullable_int': {'schema': {'type': 'list', 'items': {'type': 'nullable', 'schema': 'int'}}},
                'field_union': {
                    'schema': {
                        'type': 'union',
                        'choices': [
                            'str',
                            {
                                'type': 'model',
                                'fields': {
                                    'field_str': {'schema': 'str'},
                                    'field_int': {'schema': 'int'},
                                    'field_float': {'schema': 'float'},
                                },
                            },
                            {
                                'type': 'model',
                                'fields': {
                                    'field_float': {'schema': 'float'},
                                    'field_bytes': {'schema': 'bytes'},
                                    'field_date': {'schema': 'date'},
                                },
                            },
                        ],
                    }
                },
                'field_functions_model': {
                    'schema': {
                        'type': 'model',
                        'fields': {
                            'field_before': {
                                'schema': {
                                    'type': 'function',
                                    'mode': 'before',
                                    'function': append_func,
                                    'schema': {'type': 'str'},
                                }
                            },
                            'field_after': {
                                'schema': {
                                    'type': 'function',
                                    'mode': 'after',
                                    'function': append_func,
                                    'schema': {'type': 'str'},
                                }
                            },
                            'field_wrap': {
                                'schema': {
                                    'type': 'function',
                                    'mode': 'wrap',
                                    'function': wrap_function,
                                    'schema': {'type': 'str'},
                                }
                            },
                            'field_plain': {'schema': {'type': 'function', 'mode': 'plain', 'function': append_func}},
                        },
                    }
                },
                'field_recursive': {
                    'schema': {
                        'type': 'recursive-container',
                        'name': 'Branch',
                        'schema': {
                            'type': 'model',
                            'fields': {
                                'name': {'schema': 'str'},
                                'sub_branch': {
                                    'schema': {
                                        'type': 'nullable',
                                        'schema': {'type': 'recursive-ref', 'name': 'Branch'},
                                    },
                                    'default': None,
                                },
                            },
                        },
                    }
                },
            },
        },
    }


def complete_pydantic_model():
    from typing import Literal, Any, Union
    from datetime import date, time, datetime

    try:
        from pydantic import BaseModel, validator, constr, conint, confloat, conbytes, conlist, conset, confrozenset
    except ImportError:
        return None

    class UnionModel1(BaseModel):
        field_str: str
        field_int: int
        field_float: float

    class UnionModel2(BaseModel):
        field_float: float
        field_bytes: bytes
        field_date: date

    class FunctionModel(BaseModel):
        field_before: str
        field_after: str
        field_wrap: str
        field_plain: Any

        @validator('field_before', pre=True, allow_reuse=True)
        def append_before(cls, v):
            return f'{v} Changed'

        @validator('field_after', 'field_wrap', 'field_plain', allow_reuse=True)  # best attempts at wrap and plain
        def append_after(cls, v):
            return f'{v} Changed'

        @validator('field_wrap', pre=True, allow_reuse=True)  # other part of wrap
        def wrap_before(cls, v):
            return f'Input {v}'

    class BranchModel(BaseModel):
        name: str
        sub_branch: 'BranchModel' = None

    class Model(BaseModel):
        field_str: str
        field_str_con: constr(min_length=3, max_length=5, regex='^[a-z]+$')
        field_int: int
        field_int_con: conint(gt=1, lt=10, multiple_of=2)
        field_float: float
        field_float_con: confloat(ge=1.0, le=10.0, multiple_of=0.5)
        field_bool: bool
        field_bytes: bytes
        field_bytes_con: conbytes(min_length=6, max_length=1000)
        field_date: date
        field_date_con: date  # todo ge='2020-01-01', lt='2020-01-02'
        field_time: time
        field_time_con: time  # todo ge='06:00:00', lt='12:13:14'
        field_datetime: datetime
        field_datetime_con: datetime  # todo ge='2000-01-01T06:00:00', lt='2020-01-02T12:13:14'
        field_list_any: list
        field_list_str: list[str]
        field_list_str_con: conlist(str, min_items=3, max_items=42)
        field_set_any: set
        field_set_int: set[int]
        field_set_int_con: conset(int, min_items=3, max_items=42)
        field_frozenset_any: frozenset
        field_frozenset_bytes: frozenset[bytes]
        field_frozenset_bytes_con: confrozenset(bytes, min_items=3, max_items=42)
        field_tuple_var_len_any: tuple[Any, ...]
        field_tuple_var_len_float: tuple[float, ...]
        field_tuple_var_len_float_con: tuple[float, ...]  # todo min_items=3, max_items=42
        field_tuple_fix_len: tuple[str, int, float, bool]
        field_dict_any: dict
        field_dict_str_float: dict[str, float]
        field_literal_1_int: Literal[1]
        field_literal_1_str: Literal['foobar']
        field_literal_mult_int: Literal[1, 2, 3]
        field_literal_mult_str: Literal['foo', 'bar', 'baz']
        field_literal_assorted: Literal[1, 'foo', True]
        field_list_nullable_int: list[int | None]
        field_union: Union[str, UnionModel1, UnionModel2]
        field_functions_model: FunctionModel
        field_recursive: BranchModel

    return Model


def complete_input_data():
    return {
        'field_str': 'fo',
        'field_str_con': 'fooba',
        'field_int': 1,
        'field_int_con': 8,
        'field_float': 1.0,
        'field_float_con': 10.0,
        'field_bool': True,
        'field_bytes': b'foobar',
        'field_bytes_con': b'foobar',
        'field_date': '2020-01-01',
        'field_date_con': '2020-01-01',
        'field_time': '12:00:00',
        'field_time_con': '12:00:00',
        'field_datetime': '2020-01-01T00:00:00',
        'field_datetime_con': '2020-01-01T00:00:00',
        'field_list_any': ['a', b'b', True, 1.0, None],
        'field_list_str': ['a', 'b', 'c'],
        'field_list_str_con': ['a', 'b', 'c'],
        'field_set_any': {'a', b'b', True, 1.0, None},
        'field_set_int': {1, 2, 3},
        'field_set_int_con': {1, 2, 3},
        'field_frozenset_any': frozenset({'a', b'b', True, 1.0, None}),
        'field_frozenset_bytes': frozenset({b'a', b'b', b'c'}),
        'field_frozenset_bytes_con': frozenset({b'a', b'b', b'c'}),
        'field_tuple_var_len_any': ('a', b'b', True, 1.0, None),
        'field_tuple_var_len_float': (1.0, 2.0, 3.0),
        'field_tuple_var_len_float_con': (1.0, 2.0, 3.0),
        'field_tuple_fix_len': ('a', 1, 1.0, True),
        'field_dict_any': {'a': 'b', 1: True, 1.0: 1.0},
        'field_dict_str_float': {'a': 1.0, 'b': 2.0, 'c': 3.0},
        'field_literal_1_int': 1,
        'field_literal_1_str': 'foobar',
        'field_literal_mult_int': 3,
        'field_literal_mult_str': 'foo',
        'field_literal_assorted': 'foo',
        'field_list_nullable_int': [1, None, 2, None, 3, None, 4, None],
        'field_union': {'field_str': 'foo', 'field_int': 1, 'field_float': 1.0},
        'field_functions_model': {
            'field_before': 'foo',
            'field_after': 'foo',
            'field_wrap': 'foo',
            'field_plain': 'foo',
        },
        'field_recursive': {
            'name': 'foo',
            'sub_branch': {'name': 'bar', 'sub_branch': {'name': 'baz', 'sub_branch': None}},
        },
    }
