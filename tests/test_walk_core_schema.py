from copy import deepcopy
from typing import Any, Callable, Generic, TypeVar

from pydantic_core import WalkCoreSchema
from pydantic_core import core_schema as cs
from pydantic_core.core_schema import CoreSchema, SerSchema

CoreSchemaCallNext = Callable[[CoreSchema], CoreSchema]
SerSchemaCallNext = Callable[[SerSchema], SerSchema]

CallableF = TypeVar('CallableF', bound=Callable[..., Any])


class NamedFunction(Generic[CallableF]):
    def __init__(self, func: CallableF) -> None:
        self.func = func

    def __call__(self, *args: Any, **kwargs: Any) -> Any:
        return self.func(*args, **kwargs)

    def __repr__(self) -> str:
        return f'NamedFunction({self.func.__name__})'

    def __eq__(self, other: Any) -> bool:
        if not isinstance(other, NamedFunction):
            return False
        return self.func is other.func  # type: ignore


class SimpleRepr(type):
    def __repr__(cls):
        return cls.__name__


class NamedClass(metaclass=SimpleRepr):
    pass


def _plain_ser_func(x: Any) -> str:
    return 'abc'


plain_ser_func = NamedFunction(_plain_ser_func)


def _no_info_val_func(x: Any) -> Any:
    return x


no_info_val_func = NamedFunction(_no_info_val_func)


def _no_info_wrap_val_func(x: Any, handler: cs.ValidatorFunctionWrapHandler) -> Any:
    return handler(x)


no_info_wrap_val_func = NamedFunction(_no_info_wrap_val_func)


def test_walk_core_schema_before():
    called: list[CoreSchema | SerSchema] = []

    def cs_handler(schema: CoreSchema, call_next: CoreSchemaCallNext) -> CoreSchema:
        called.append(deepcopy(schema))
        old = deepcopy(schema)
        try:
            new = call_next(schema)
            assert new == old
            return new
        except Exception as e:
            print(e)
            print(schema['type'])
            raise

    def ser_handler(schema: SerSchema, call_next: SerSchemaCallNext) -> SerSchema:
        called.append(deepcopy(schema))
        return call_next(schema)

    walk = WalkCoreSchema(
        visit_any_schema=cs_handler,
        visit_none_schema=cs_handler,
        visit_bool_schema=cs_handler,
        visit_int_schema=cs_handler,
        visit_float_schema=cs_handler,
        visit_decimal_schema=cs_handler,
        visit_string_schema=cs_handler,
        visit_bytes_schema=cs_handler,
        visit_date_schema=cs_handler,
        visit_time_schema=cs_handler,
        visit_datetime_schema=cs_handler,
        visit_timedelta_schema=cs_handler,
        visit_literal_schema=cs_handler,
        visit_is_instance_schema=cs_handler,
        visit_is_subclass_schema=cs_handler,
        visit_callable_schema=cs_handler,
        visit_list_schema=cs_handler,
        visit_tuple_positional_schema=cs_handler,
        visit_tuple_variable_schema=cs_handler,
        visit_set_schema=cs_handler,
        visit_frozenset_schema=cs_handler,
        visit_generator_schema=cs_handler,
        visit_dict_schema=cs_handler,
        visit_after_validator_function_schema=cs_handler,
        visit_before_validator_function_schema=cs_handler,
        visit_wrap_validator_function_schema=cs_handler,
        visit_plain_validator_function_schema=cs_handler,
        visit_with_default_schema=cs_handler,
        visit_nullable_schema=cs_handler,
        visit_union_schema=cs_handler,
        visit_tagged_union_schema=cs_handler,
        visit_chain_schema=cs_handler,
        visit_lax_or_strict_schema=cs_handler,
        visit_json_or_python_schema=cs_handler,
        visit_typed_dict_schema=cs_handler,
        visit_model_fields_schema=cs_handler,
        visit_model_schema=cs_handler,
        visit_dataclass_args_schema=cs_handler,
        visit_dataclass_schema=cs_handler,
        visit_arguments_schema=cs_handler,
        visit_call_schema=cs_handler,
        visit_custom_error_schema=cs_handler,
        visit_json_schema=cs_handler,
        visit_url_schema=cs_handler,
        visit_multi_host_url_schema=cs_handler,
        visit_definitions_schema=cs_handler,
        visit_definition_reference_schema=cs_handler,
        visit_uuid_schema=cs_handler,
        # ser schemas, see SerSchema in core_schema.py
        visit_plain_function_ser_schema=ser_handler,
        visit_wrap_function_ser_schema=ser_handler,
        visit_format_ser_schema=ser_handler,
        visit_to_string_ser_schema=ser_handler,
        visit_model_ser_schema=ser_handler,
    )

    schema = cs.union_schema(
        [
            cs.any_schema(serialization=cs.plain_serializer_function_ser_schema(plain_ser_func)),
            cs.none_schema(serialization=cs.plain_serializer_function_ser_schema(plain_ser_func)),
            cs.bool_schema(serialization=cs.simple_ser_schema('bool')),
            cs.int_schema(serialization=cs.simple_ser_schema('int')),
            cs.float_schema(serialization=cs.simple_ser_schema('float')),
            cs.decimal_schema(serialization=cs.plain_serializer_function_ser_schema(plain_ser_func)),
            cs.str_schema(serialization=cs.simple_ser_schema('str')),
            cs.bytes_schema(serialization=cs.simple_ser_schema('bytes')),
            cs.date_schema(serialization=cs.simple_ser_schema('date')),
            cs.time_schema(serialization=cs.simple_ser_schema('time')),
            cs.datetime_schema(serialization=cs.simple_ser_schema('datetime')),
            cs.timedelta_schema(serialization=cs.simple_ser_schema('timedelta')),
            cs.literal_schema(
                expected=[1, 2, 3],
                serialization=cs.plain_serializer_function_ser_schema(plain_ser_func),
            ),
            cs.is_instance_schema(
                cls=NamedClass,
                serialization=cs.plain_serializer_function_ser_schema(plain_ser_func),
            ),
            cs.is_subclass_schema(
                cls=NamedClass,
                serialization=cs.plain_serializer_function_ser_schema(plain_ser_func),
            ),
            cs.callable_schema(
                serialization=cs.plain_serializer_function_ser_schema(plain_ser_func),
            ),
            cs.list_schema(
                cs.int_schema(serialization=cs.simple_ser_schema('int')),
                serialization=cs.plain_serializer_function_ser_schema(plain_ser_func),
            ),
            cs.tuple_positional_schema(
                [cs.int_schema(serialization=cs.simple_ser_schema('int'))],
                extras_schema=cs.int_schema(serialization=cs.simple_ser_schema('int')),
                serialization=cs.plain_serializer_function_ser_schema(plain_ser_func),
            ),
            cs.tuple_variable_schema(
                cs.int_schema(serialization=cs.simple_ser_schema('int')),
                serialization=cs.plain_serializer_function_ser_schema(plain_ser_func),
            ),
            cs.set_schema(
                cs.int_schema(serialization=cs.simple_ser_schema('int')),
                serialization=cs.plain_serializer_function_ser_schema(plain_ser_func),
            ),
            cs.frozenset_schema(
                cs.int_schema(serialization=cs.simple_ser_schema('int')),
                serialization=cs.plain_serializer_function_ser_schema(plain_ser_func),
            ),
            cs.generator_schema(
                cs.int_schema(serialization=cs.simple_ser_schema('int')),
                serialization=cs.plain_serializer_function_ser_schema(plain_ser_func),
            ),
            cs.dict_schema(
                cs.int_schema(serialization=cs.simple_ser_schema('int')),
                cs.int_schema(serialization=cs.simple_ser_schema('int')),
                serialization=cs.plain_serializer_function_ser_schema(plain_ser_func),
            ),
            cs.no_info_after_validator_function(
                no_info_val_func,
                cs.int_schema(),
                serialization=cs.plain_serializer_function_ser_schema(plain_ser_func),
            ),
            cs.no_info_before_validator_function(
                no_info_val_func,
                cs.int_schema(),
                serialization=cs.plain_serializer_function_ser_schema(plain_ser_func),
            ),
            cs.no_info_wrap_validator_function(
                no_info_wrap_val_func,
                cs.int_schema(),
                serialization=cs.plain_serializer_function_ser_schema(plain_ser_func),
            ),
            cs.no_info_plain_validator_function(
                no_info_val_func,
                serialization=cs.plain_serializer_function_ser_schema(plain_ser_func),
            ),
            cs.with_default_schema(
                cs.int_schema(),
                default=1,
                serialization=cs.plain_serializer_function_ser_schema(plain_ser_func),
            ),
            cs.nullable_schema(
                cs.int_schema(),
                serialization=cs.plain_serializer_function_ser_schema(plain_ser_func),
            ),
            cs.union_schema(
                [
                    cs.int_schema(),
                    cs.str_schema(),
                ],
                serialization=cs.plain_serializer_function_ser_schema(plain_ser_func),
            ),
            cs.tagged_union_schema(
                {
                    'a': cs.int_schema(),
                    'b': cs.str_schema(),
                },
                'type',
                serialization=cs.plain_serializer_function_ser_schema(plain_ser_func),
            ),
            cs.chain_schema(
                [
                    cs.int_schema(),
                    cs.str_schema(),
                ],
                serialization=cs.plain_serializer_function_ser_schema(plain_ser_func),
            ),
            cs.lax_or_strict_schema(
                cs.int_schema(),
                cs.str_schema(),
                serialization=cs.plain_serializer_function_ser_schema(plain_ser_func),
            ),
            cs.json_or_python_schema(
                cs.int_schema(),
                cs.str_schema(),
                serialization=cs.plain_serializer_function_ser_schema(plain_ser_func),
            ),
            cs.typed_dict_schema(
                {'a': cs.typed_dict_field(cs.int_schema())},
                computed_fields=[
                    cs.computed_field(
                        'b',
                        cs.int_schema(),
                    )
                ],
                extras_schema=cs.int_schema(serialization=cs.simple_ser_schema('int')),
                serialization=cs.plain_serializer_function_ser_schema(plain_ser_func),
            ),
            cs.model_schema(
                NamedClass,
                cs.model_fields_schema(
                    {'a': cs.model_field(cs.int_schema())},
                    extras_schema=cs.int_schema(serialization=cs.simple_ser_schema('int')),
                    computed_fields=[
                        cs.computed_field(
                            'b',
                            cs.int_schema(),
                        )
                    ],
                ),
            ),
            cs.dataclass_schema(
                NamedClass,
                cs.dataclass_args_schema(
                    'Model',
                    [cs.dataclass_field('a', cs.int_schema())],
                    computed_fields=[
                        cs.computed_field(
                            'b',
                            cs.int_schema(),
                        )
                    ],
                ),
                ['a'],
            ),
            cs.call_schema(
                cs.arguments_schema(
                    [cs.arguments_parameter('x', cs.int_schema())],
                    serialization=cs.plain_serializer_function_ser_schema(plain_ser_func),
                ),
                no_info_val_func,
                serialization=cs.plain_serializer_function_ser_schema(plain_ser_func),
            ),
            cs.custom_error_schema(
                cs.int_schema(),
                custom_error_type='CustomError',
                serialization=cs.plain_serializer_function_ser_schema(plain_ser_func),
            ),
            cs.json_schema(
                cs.int_schema(),
                serialization=cs.plain_serializer_function_ser_schema(plain_ser_func),
            ),
            cs.url_schema(
                serialization=cs.plain_serializer_function_ser_schema(plain_ser_func),
            ),
            cs.multi_host_url_schema(
                serialization=cs.plain_serializer_function_ser_schema(plain_ser_func),
            ),
            cs.definitions_schema(
                cs.int_schema(),
                [
                    cs.int_schema(ref='#/definitions/int'),
                ],
            ),
            cs.definition_reference_schema(
                '#/definitions/int',
                serialization=cs.plain_serializer_function_ser_schema(plain_ser_func),
            ),
            cs.uuid_schema(
                serialization=cs.plain_serializer_function_ser_schema(plain_ser_func),
            ),
        ]
    )

    walk.walk(schema)

    # insert_assert(called)
    assert called == [
        {
            'type': 'union',
            'choices': [
                {
                    'type': 'any',
                    'serialization': {
                        'type': 'function-plain',
                        'function': NamedFunction(_plain_ser_func),
                    },
                },
                {
                    'type': 'none',
                    'serialization': {
                        'type': 'function-plain',
                        'function': NamedFunction(_plain_ser_func),
                    },
                },
                {'type': 'bool', 'serialization': {'type': 'bool'}},
                {'type': 'int', 'serialization': {'type': 'int'}},
                {'type': 'float', 'serialization': {'type': 'float'}},
                {
                    'type': 'decimal',
                    'serialization': {
                        'type': 'function-plain',
                        'function': NamedFunction(_plain_ser_func),
                    },
                },
                {'type': 'str', 'serialization': {'type': 'str'}},
                {'type': 'bytes', 'serialization': {'type': 'bytes'}},
                {'type': 'date', 'serialization': {'type': 'date'}},
                {
                    'type': 'time',
                    'microseconds_precision': 'truncate',
                    'serialization': {'type': 'time'},
                },
                {
                    'type': 'datetime',
                    'microseconds_precision': 'truncate',
                    'serialization': {'type': 'datetime'},
                },
                {
                    'type': 'timedelta',
                    'microseconds_precision': 'truncate',
                    'serialization': {'type': 'timedelta'},
                },
                {
                    'type': 'literal',
                    'expected': [1, 2, 3],
                    'serialization': {
                        'type': 'function-plain',
                        'function': NamedFunction(_plain_ser_func),
                    },
                },
                {
                    'type': 'is-instance',
                    'cls': NamedClass,
                    'serialization': {
                        'type': 'function-plain',
                        'function': NamedFunction(_plain_ser_func),
                    },
                },
                {
                    'type': 'is-subclass',
                    'cls': NamedClass,
                    'serialization': {
                        'type': 'function-plain',
                        'function': NamedFunction(_plain_ser_func),
                    },
                },
                {
                    'type': 'callable',
                    'serialization': {
                        'type': 'function-plain',
                        'function': NamedFunction(_plain_ser_func),
                    },
                },
                {
                    'type': 'list',
                    'items_schema': {'type': 'int', 'serialization': {'type': 'int'}},
                    'serialization': {
                        'type': 'function-plain',
                        'function': NamedFunction(_plain_ser_func),
                    },
                },
                {
                    'type': 'tuple-positional',
                    'items_schema': [{'type': 'int', 'serialization': {'type': 'int'}}],
                    'extras_schema': {'type': 'int', 'serialization': {'type': 'int'}},
                    'serialization': {
                        'type': 'function-plain',
                        'function': NamedFunction(_plain_ser_func),
                    },
                },
                {
                    'type': 'tuple-variable',
                    'items_schema': {'type': 'int', 'serialization': {'type': 'int'}},
                    'serialization': {
                        'type': 'function-plain',
                        'function': NamedFunction(_plain_ser_func),
                    },
                },
                {
                    'type': 'set',
                    'items_schema': {'type': 'int', 'serialization': {'type': 'int'}},
                    'serialization': {
                        'type': 'function-plain',
                        'function': NamedFunction(_plain_ser_func),
                    },
                },
                {
                    'type': 'frozenset',
                    'items_schema': {'type': 'int', 'serialization': {'type': 'int'}},
                    'serialization': {
                        'type': 'function-plain',
                        'function': NamedFunction(_plain_ser_func),
                    },
                },
                {
                    'type': 'generator',
                    'items_schema': {'type': 'int', 'serialization': {'type': 'int'}},
                    'serialization': {
                        'type': 'function-plain',
                        'function': NamedFunction(_plain_ser_func),
                    },
                },
                {
                    'type': 'dict',
                    'keys_schema': {'type': 'int', 'serialization': {'type': 'int'}},
                    'values_schema': {'type': 'int', 'serialization': {'type': 'int'}},
                    'serialization': {
                        'type': 'function-plain',
                        'function': NamedFunction(_plain_ser_func),
                    },
                },
                {
                    'type': 'function-after',
                    'function': {
                        'type': 'no-info',
                        'function': NamedFunction(_no_info_val_func),
                    },
                    'schema': {'type': 'int'},
                    'serialization': {
                        'type': 'function-plain',
                        'function': NamedFunction(_plain_ser_func),
                    },
                },
                {
                    'type': 'function-before',
                    'function': {
                        'type': 'no-info',
                        'function': NamedFunction(_no_info_val_func),
                    },
                    'schema': {'type': 'int'},
                    'serialization': {
                        'type': 'function-plain',
                        'function': NamedFunction(_plain_ser_func),
                    },
                },
                {
                    'type': 'function-wrap',
                    'function': {
                        'type': 'no-info',
                        'function': NamedFunction(_no_info_wrap_val_func),
                    },
                    'schema': {'type': 'int'},
                    'serialization': {
                        'type': 'function-plain',
                        'function': NamedFunction(_plain_ser_func),
                    },
                },
                {
                    'type': 'function-plain',
                    'function': {
                        'type': 'no-info',
                        'function': NamedFunction(_no_info_val_func),
                    },
                    'serialization': {
                        'type': 'function-plain',
                        'function': NamedFunction(_plain_ser_func),
                    },
                },
                {
                    'type': 'default',
                    'schema': {'type': 'int'},
                    'serialization': {
                        'type': 'function-plain',
                        'function': NamedFunction(_plain_ser_func),
                    },
                    'default': 1,
                },
                {
                    'type': 'nullable',
                    'schema': {'type': 'int'},
                    'serialization': {
                        'type': 'function-plain',
                        'function': NamedFunction(_plain_ser_func),
                    },
                },
                {
                    'type': 'union',
                    'choices': [{'type': 'int'}, {'type': 'str'}],
                    'serialization': {
                        'type': 'function-plain',
                        'function': NamedFunction(_plain_ser_func),
                    },
                },
                {
                    'type': 'tagged-union',
                    'choices': {'a': {'type': 'int'}, 'b': {'type': 'str'}},
                    'discriminator': 'type',
                    'serialization': {
                        'type': 'function-plain',
                        'function': NamedFunction(_plain_ser_func),
                    },
                },
                {
                    'type': 'chain',
                    'steps': [{'type': 'int'}, {'type': 'str'}],
                    'serialization': {
                        'type': 'function-plain',
                        'function': NamedFunction(_plain_ser_func),
                    },
                },
                {
                    'type': 'lax-or-strict',
                    'lax_schema': {'type': 'int'},
                    'strict_schema': {'type': 'str'},
                    'serialization': {
                        'type': 'function-plain',
                        'function': NamedFunction(_plain_ser_func),
                    },
                },
                {
                    'type': 'json-or-python',
                    'json_schema': {'type': 'int'},
                    'python_schema': {'type': 'str'},
                    'serialization': {
                        'type': 'function-plain',
                        'function': NamedFunction(_plain_ser_func),
                    },
                },
                {
                    'type': 'typed-dict',
                    'fields': {'a': {'type': 'typed-dict-field', 'schema': {'type': 'int'}}},
                    'computed_fields': [
                        {
                            'type': 'computed-field',
                            'property_name': 'b',
                            'return_schema': {'type': 'int'},
                        }
                    ],
                    'extras_schema': {'type': 'int', 'serialization': {'type': 'int'}},
                    'serialization': {
                        'type': 'function-plain',
                        'function': NamedFunction(_plain_ser_func),
                    },
                },
                {
                    'type': 'model',
                    'cls': NamedClass,
                    'schema': {
                        'type': 'model-fields',
                        'fields': {'a': {'type': 'model-field', 'schema': {'type': 'int'}}},
                        'computed_fields': [
                            {
                                'type': 'computed-field',
                                'property_name': 'b',
                                'return_schema': {'type': 'int'},
                            }
                        ],
                        'extras_schema': {'type': 'int', 'serialization': {'type': 'int'}},
                    },
                },
                {
                    'type': 'dataclass',
                    'cls': NamedClass,
                    'fields': ['a'],
                    'schema': {
                        'type': 'dataclass-args',
                        'dataclass_name': 'Model',
                        'fields': [
                            {
                                'type': 'dataclass-field',
                                'name': 'a',
                                'schema': {'type': 'int'},
                            }
                        ],
                        'computed_fields': [
                            {
                                'type': 'computed-field',
                                'property_name': 'b',
                                'return_schema': {'type': 'int'},
                            }
                        ],
                    },
                },
                {
                    'type': 'call',
                    'arguments_schema': {
                        'type': 'arguments',
                        'arguments_schema': [{'name': 'x', 'schema': {'type': 'int'}}],
                        'serialization': {
                            'type': 'function-plain',
                            'function': NamedFunction(_plain_ser_func),
                        },
                    },
                    'function': NamedFunction(_no_info_val_func),
                    'serialization': {
                        'type': 'function-plain',
                        'function': NamedFunction(_plain_ser_func),
                    },
                },
                {
                    'type': 'custom-error',
                    'schema': {'type': 'int'},
                    'custom_error_type': 'CustomError',
                    'serialization': {
                        'type': 'function-plain',
                        'function': NamedFunction(_plain_ser_func),
                    },
                },
                {
                    'type': 'json',
                    'schema': {'type': 'int'},
                    'serialization': {
                        'type': 'function-plain',
                        'function': NamedFunction(_plain_ser_func),
                    },
                },
                {
                    'type': 'url',
                    'serialization': {
                        'type': 'function-plain',
                        'function': NamedFunction(_plain_ser_func),
                    },
                },
                {
                    'type': 'multi-host-url',
                    'serialization': {
                        'type': 'function-plain',
                        'function': NamedFunction(_plain_ser_func),
                    },
                },
                {
                    'type': 'definitions',
                    'schema': {'type': 'int'},
                    'definitions': [{'type': 'int', 'ref': '#/definitions/int'}],
                },
                {
                    'type': 'definition-ref',
                    'schema_ref': '#/definitions/int',
                    'serialization': {
                        'type': 'function-plain',
                        'function': NamedFunction(_plain_ser_func),
                    },
                },
                {
                    'type': 'uuid',
                    'serialization': {
                        'type': 'function-plain',
                        'function': NamedFunction(_plain_ser_func),
                    },
                },
            ],
        },
        {
            'type': 'any',
            'serialization': {
                'type': 'function-plain',
                'function': NamedFunction(_plain_ser_func),
            },
        },
        {'type': 'function-plain', 'function': NamedFunction(_plain_ser_func)},
        {
            'type': 'none',
            'serialization': {
                'type': 'function-plain',
                'function': NamedFunction(_plain_ser_func),
            },
        },
        {'type': 'function-plain', 'function': NamedFunction(_plain_ser_func)},
        {'type': 'bool', 'serialization': {'type': 'bool'}},
        {'type': 'bool'},
        {'type': 'int', 'serialization': {'type': 'int'}},
        {'type': 'int'},
        {'type': 'float', 'serialization': {'type': 'float'}},
        {'type': 'float'},
        {
            'type': 'decimal',
            'serialization': {
                'type': 'function-plain',
                'function': NamedFunction(_plain_ser_func),
            },
        },
        {'type': 'function-plain', 'function': NamedFunction(_plain_ser_func)},
        {'type': 'str', 'serialization': {'type': 'str'}},
        {'type': 'str'},
        {'type': 'bytes', 'serialization': {'type': 'bytes'}},
        {'type': 'bytes'},
        {'type': 'date', 'serialization': {'type': 'date'}},
        {'type': 'date'},
        {
            'type': 'time',
            'microseconds_precision': 'truncate',
            'serialization': {'type': 'time'},
        },
        {'type': 'time'},
        {
            'type': 'datetime',
            'microseconds_precision': 'truncate',
            'serialization': {'type': 'datetime'},
        },
        {'type': 'datetime'},
        {
            'type': 'timedelta',
            'microseconds_precision': 'truncate',
            'serialization': {'type': 'timedelta'},
        },
        {'type': 'timedelta'},
        {
            'type': 'literal',
            'expected': [1, 2, 3],
            'serialization': {
                'type': 'function-plain',
                'function': NamedFunction(_plain_ser_func),
            },
        },
        {'type': 'function-plain', 'function': NamedFunction(_plain_ser_func)},
        {
            'type': 'is-instance',
            'cls': NamedClass,
            'serialization': {
                'type': 'function-plain',
                'function': NamedFunction(_plain_ser_func),
            },
        },
        {'type': 'function-plain', 'function': NamedFunction(_plain_ser_func)},
        {
            'type': 'is-subclass',
            'cls': NamedClass,
            'serialization': {
                'type': 'function-plain',
                'function': NamedFunction(_plain_ser_func),
            },
        },
        {'type': 'function-plain', 'function': NamedFunction(_plain_ser_func)},
        {
            'type': 'callable',
            'serialization': {
                'type': 'function-plain',
                'function': NamedFunction(_plain_ser_func),
            },
        },
        {'type': 'function-plain', 'function': NamedFunction(_plain_ser_func)},
        {
            'type': 'list',
            'items_schema': {'type': 'int', 'serialization': {'type': 'int'}},
            'serialization': {
                'type': 'function-plain',
                'function': NamedFunction(_plain_ser_func),
            },
        },
        {'type': 'int', 'serialization': {'type': 'int'}},
        {'type': 'int'},
        {'type': 'function-plain', 'function': NamedFunction(_plain_ser_func)},
        {
            'type': 'tuple-positional',
            'items_schema': [{'type': 'int', 'serialization': {'type': 'int'}}],
            'extras_schema': {'type': 'int', 'serialization': {'type': 'int'}},
            'serialization': {
                'type': 'function-plain',
                'function': NamedFunction(_plain_ser_func),
            },
        },
        {'type': 'int', 'serialization': {'type': 'int'}},
        {'type': 'int'},
        {'type': 'int', 'serialization': {'type': 'int'}},
        {'type': 'int'},
        {'type': 'function-plain', 'function': NamedFunction(_plain_ser_func)},
        {
            'type': 'tuple-variable',
            'items_schema': {'type': 'int', 'serialization': {'type': 'int'}},
            'serialization': {
                'type': 'function-plain',
                'function': NamedFunction(_plain_ser_func),
            },
        },
        {'type': 'int', 'serialization': {'type': 'int'}},
        {'type': 'int'},
        {'type': 'function-plain', 'function': NamedFunction(_plain_ser_func)},
        {
            'type': 'set',
            'items_schema': {'type': 'int', 'serialization': {'type': 'int'}},
            'serialization': {
                'type': 'function-plain',
                'function': NamedFunction(_plain_ser_func),
            },
        },
        {'type': 'int', 'serialization': {'type': 'int'}},
        {'type': 'int'},
        {'type': 'function-plain', 'function': NamedFunction(_plain_ser_func)},
        {
            'type': 'frozenset',
            'items_schema': {'type': 'int', 'serialization': {'type': 'int'}},
            'serialization': {
                'type': 'function-plain',
                'function': NamedFunction(_plain_ser_func),
            },
        },
        {'type': 'int', 'serialization': {'type': 'int'}},
        {'type': 'int'},
        {'type': 'function-plain', 'function': NamedFunction(_plain_ser_func)},
        {
            'type': 'generator',
            'items_schema': {'type': 'int', 'serialization': {'type': 'int'}},
            'serialization': {
                'type': 'function-plain',
                'function': NamedFunction(_plain_ser_func),
            },
        },
        {'type': 'int', 'serialization': {'type': 'int'}},
        {'type': 'int'},
        {'type': 'function-plain', 'function': NamedFunction(_plain_ser_func)},
        {
            'type': 'dict',
            'keys_schema': {'type': 'int', 'serialization': {'type': 'int'}},
            'values_schema': {'type': 'int', 'serialization': {'type': 'int'}},
            'serialization': {
                'type': 'function-plain',
                'function': NamedFunction(_plain_ser_func),
            },
        },
        {'type': 'int', 'serialization': {'type': 'int'}},
        {'type': 'int'},
        {'type': 'int', 'serialization': {'type': 'int'}},
        {'type': 'int'},
        {'type': 'function-plain', 'function': NamedFunction(_plain_ser_func)},
        {
            'type': 'function-after',
            'function': {'type': 'no-info', 'function': NamedFunction(_no_info_val_func)},
            'schema': {'type': 'int'},
            'serialization': {
                'type': 'function-plain',
                'function': NamedFunction(_plain_ser_func),
            },
        },
        {'type': 'function-plain', 'function': NamedFunction(_plain_ser_func)},
        {
            'type': 'function-before',
            'function': {'type': 'no-info', 'function': NamedFunction(_no_info_val_func)},
            'schema': {'type': 'int'},
            'serialization': {
                'type': 'function-plain',
                'function': NamedFunction(_plain_ser_func),
            },
        },
        {'type': 'function-plain', 'function': NamedFunction(_plain_ser_func)},
        {
            'type': 'function-wrap',
            'function': {
                'type': 'no-info',
                'function': NamedFunction(_no_info_wrap_val_func),
            },
            'schema': {'type': 'int'},
            'serialization': {
                'type': 'function-plain',
                'function': NamedFunction(_plain_ser_func),
            },
        },
        {'type': 'int'},
        {'type': 'function-plain', 'function': NamedFunction(_plain_ser_func)},
        {
            'type': 'function-plain',
            'function': {'type': 'no-info', 'function': NamedFunction(_no_info_val_func)},
            'serialization': {
                'type': 'function-plain',
                'function': NamedFunction(_plain_ser_func),
            },
        },
        {'type': 'function-plain', 'function': NamedFunction(_plain_ser_func)},
        {
            'type': 'default',
            'schema': {'type': 'int'},
            'serialization': {
                'type': 'function-plain',
                'function': NamedFunction(_plain_ser_func),
            },
            'default': 1,
        },
        {'type': 'int'},
        {'type': 'function-plain', 'function': NamedFunction(_plain_ser_func)},
        {
            'type': 'nullable',
            'schema': {'type': 'int'},
            'serialization': {
                'type': 'function-plain',
                'function': NamedFunction(_plain_ser_func),
            },
        },
        {'type': 'int'},
        {'type': 'function-plain', 'function': NamedFunction(_plain_ser_func)},
        {
            'type': 'union',
            'choices': [{'type': 'int'}, {'type': 'str'}],
            'serialization': {
                'type': 'function-plain',
                'function': NamedFunction(_plain_ser_func),
            },
        },
        {'type': 'int'},
        {'type': 'str'},
        {'type': 'function-plain', 'function': NamedFunction(_plain_ser_func)},
        {
            'type': 'tagged-union',
            'choices': {'a': {'type': 'int'}, 'b': {'type': 'str'}},
            'discriminator': 'type',
            'serialization': {
                'type': 'function-plain',
                'function': NamedFunction(_plain_ser_func),
            },
        },
        {'type': 'int'},
        {'type': 'str'},
        {'type': 'function-plain', 'function': NamedFunction(_plain_ser_func)},
        {
            'type': 'chain',
            'steps': [{'type': 'int'}, {'type': 'str'}],
            'serialization': {
                'type': 'function-plain',
                'function': NamedFunction(_plain_ser_func),
            },
        },
        {'type': 'int'},
        {'type': 'str'},
        {'type': 'function-plain', 'function': NamedFunction(_plain_ser_func)},
        {
            'type': 'lax-or-strict',
            'lax_schema': {'type': 'int'},
            'strict_schema': {'type': 'str'},
            'serialization': {
                'type': 'function-plain',
                'function': NamedFunction(_plain_ser_func),
            },
        },
        {'type': 'int'},
        {'type': 'str'},
        {'type': 'function-plain', 'function': NamedFunction(_plain_ser_func)},
        {
            'type': 'json-or-python',
            'json_schema': {'type': 'int'},
            'python_schema': {'type': 'str'},
            'serialization': {
                'type': 'function-plain',
                'function': NamedFunction(_plain_ser_func),
            },
        },
        {'type': 'int'},
        {'type': 'str'},
        {'type': 'function-plain', 'function': NamedFunction(_plain_ser_func)},
        {
            'type': 'typed-dict',
            'fields': {'a': {'type': 'typed-dict-field', 'schema': {'type': 'int'}}},
            'computed_fields': [
                {
                    'type': 'computed-field',
                    'property_name': 'b',
                    'return_schema': {'type': 'int'},
                }
            ],
            'extras_schema': {'type': 'int', 'serialization': {'type': 'int'}},
            'serialization': {
                'type': 'function-plain',
                'function': NamedFunction(_plain_ser_func),
            },
        },
        {'type': 'int'},
        {'type': 'int', 'serialization': {'type': 'int'}},
        {'type': 'int'},
        {'type': 'function-plain', 'function': NamedFunction(_plain_ser_func)},
        {
            'type': 'model',
            'cls': NamedClass,
            'schema': {
                'type': 'model-fields',
                'fields': {'a': {'type': 'model-field', 'schema': {'type': 'int'}}},
                'computed_fields': [
                    {
                        'type': 'computed-field',
                        'property_name': 'b',
                        'return_schema': {'type': 'int'},
                    }
                ],
                'extras_schema': {'type': 'int', 'serialization': {'type': 'int'}},
            },
        },
        {
            'type': 'dataclass',
            'cls': NamedClass,
            'fields': ['a'],
            'schema': {
                'type': 'dataclass-args',
                'dataclass_name': 'Model',
                'fields': [{'type': 'dataclass-field', 'name': 'a', 'schema': {'type': 'int'}}],
                'computed_fields': [
                    {
                        'type': 'computed-field',
                        'property_name': 'b',
                        'return_schema': {'type': 'int'},
                    }
                ],
            },
        },
        {
            'type': 'dataclass-args',
            'dataclass_name': 'Model',
            'fields': [{'type': 'dataclass-field', 'name': 'a', 'schema': {'type': 'int'}}],
            'computed_fields': [
                {
                    'type': 'computed-field',
                    'property_name': 'b',
                    'return_schema': {'type': 'int'},
                }
            ],
        },
        {'type': 'int'},
        {
            'type': 'call',
            'arguments_schema': {
                'type': 'arguments',
                'arguments_schema': [{'name': 'x', 'schema': {'type': 'int'}}],
                'serialization': {
                    'type': 'function-plain',
                    'function': NamedFunction(_plain_ser_func),
                },
            },
            'function': NamedFunction(_no_info_val_func),
            'serialization': {
                'type': 'function-plain',
                'function': NamedFunction(_plain_ser_func),
            },
        },
        {
            'type': 'arguments',
            'arguments_schema': [{'name': 'x', 'schema': {'type': 'int'}}],
            'serialization': {
                'type': 'function-plain',
                'function': NamedFunction(_plain_ser_func),
            },
        },
        {'type': 'function-plain', 'function': NamedFunction(_plain_ser_func)},
        {'type': 'function-plain', 'function': NamedFunction(_plain_ser_func)},
        {
            'type': 'custom-error',
            'schema': {'type': 'int'},
            'custom_error_type': 'CustomError',
            'serialization': {
                'type': 'function-plain',
                'function': NamedFunction(_plain_ser_func),
            },
        },
        {'type': 'int'},
        {'type': 'function-plain', 'function': NamedFunction(_plain_ser_func)},
        {
            'type': 'json',
            'schema': {'type': 'int'},
            'serialization': {
                'type': 'function-plain',
                'function': NamedFunction(_plain_ser_func),
            },
        },
        {'type': 'int'},
        {'type': 'function-plain', 'function': NamedFunction(_plain_ser_func)},
        {
            'type': 'url',
            'serialization': {
                'type': 'function-plain',
                'function': NamedFunction(_plain_ser_func),
            },
        },
        {'type': 'function-plain', 'function': NamedFunction(_plain_ser_func)},
        {
            'type': 'multi-host-url',
            'serialization': {
                'type': 'function-plain',
                'function': NamedFunction(_plain_ser_func),
            },
        },
        {'type': 'function-plain', 'function': NamedFunction(_plain_ser_func)},
        {
            'type': 'definitions',
            'schema': {'type': 'int'},
            'definitions': [{'type': 'int', 'ref': '#/definitions/int'}],
        },
        {'type': 'int', 'ref': '#/definitions/int'},
        {'type': 'int'},
        {
            'type': 'definition-ref',
            'schema_ref': '#/definitions/int',
            'serialization': {
                'type': 'function-plain',
                'function': NamedFunction(_plain_ser_func),
            },
        },
        {'type': 'function-plain', 'function': NamedFunction(_plain_ser_func)},
        {
            'type': 'uuid',
            'serialization': {
                'type': 'function-plain',
                'function': NamedFunction(_plain_ser_func),
            },
        },
        {'type': 'function-plain', 'function': NamedFunction(_plain_ser_func)},
    ]
