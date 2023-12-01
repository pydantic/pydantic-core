from copy import deepcopy
from dataclasses import dataclass, field
from typing import Any, Callable, Generic, TypeVar

from pydantic_core import WalkCoreSchema, WalkCoreSchemaFilterBuilder
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


def _wrap_ser_func(x: Any, handler: cs.SerializerFunctionWrapHandler) -> Any:
    return handler(x)


wrap_ser_func = NamedFunction(_wrap_ser_func)


def _no_info_val_func(x: Any) -> Any:
    return x


no_info_val_func = NamedFunction(_no_info_val_func)


def _no_info_wrap_val_func(x: Any, handler: cs.ValidatorFunctionWrapHandler) -> Any:
    return handler(x)


no_info_wrap_val_func = NamedFunction(_no_info_wrap_val_func)


SchemaT = TypeVar('SchemaT', bound=CoreSchema | SerSchema)


@dataclass
class TrackingHandler:
    called: list[str] = field(default_factory=list)
    stack: list[str] = field(default_factory=list)

    def __call__(self, schema: SchemaT, call_next: Callable[[SchemaT], SchemaT]) -> SchemaT:
        self.stack.append(schema['type'])
        self.called.append(' -> '.join(self.stack))
        old = deepcopy(schema)
        try:
            new = call_next(schema)
            assert new == old
            return new
        except Exception as e:
            print(e)
            print(schema['type'])
            raise
        finally:
            self.stack.pop()


def test_walk_core_schema_before():
    handler = TrackingHandler()

    def match_any_predicate(schema: CoreSchema | SerSchema) -> bool:
        return True

    walk = WalkCoreSchema(
        visit_core_schema=WalkCoreSchemaFilterBuilder.predicate(match_any_predicate).build(handler),
        visit_ser_schema=WalkCoreSchemaFilterBuilder.predicate(match_any_predicate).build(handler),
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

    # insert_assert(handler.called)
    assert handler.called == [
        'union',
        'union -> any',
        'union -> any -> function-plain',
        'union -> none',
        'union -> none -> function-plain',
        'union -> bool',
        'union -> bool -> bool',
        'union -> int',
        'union -> int -> int',
        'union -> float',
        'union -> float -> float',
        'union -> decimal',
        'union -> decimal -> function-plain',
        'union -> str',
        'union -> str -> str',
        'union -> bytes',
        'union -> bytes -> bytes',
        'union -> date',
        'union -> date -> date',
        'union -> time',
        'union -> time -> time',
        'union -> datetime',
        'union -> datetime -> datetime',
        'union -> timedelta',
        'union -> timedelta -> timedelta',
        'union -> literal',
        'union -> literal -> function-plain',
        'union -> is-instance',
        'union -> is-instance -> function-plain',
        'union -> is-subclass',
        'union -> is-subclass -> function-plain',
        'union -> callable',
        'union -> callable -> function-plain',
        'union -> list',
        'union -> list -> int',
        'union -> list -> int -> int',
        'union -> list -> function-plain',
        'union -> tuple-positional',
        'union -> tuple-positional -> int',
        'union -> tuple-positional -> int -> int',
        'union -> tuple-positional -> int',
        'union -> tuple-positional -> int -> int',
        'union -> tuple-positional -> function-plain',
        'union -> tuple-variable',
        'union -> tuple-variable -> int',
        'union -> tuple-variable -> int -> int',
        'union -> tuple-variable -> function-plain',
        'union -> set',
        'union -> set -> int',
        'union -> set -> int -> int',
        'union -> set -> function-plain',
        'union -> frozenset',
        'union -> frozenset -> int',
        'union -> frozenset -> int -> int',
        'union -> frozenset -> function-plain',
        'union -> generator',
        'union -> generator -> int',
        'union -> generator -> int -> int',
        'union -> generator -> function-plain',
        'union -> dict',
        'union -> dict -> int',
        'union -> dict -> int -> int',
        'union -> dict -> int',
        'union -> dict -> int -> int',
        'union -> dict -> function-plain',
        'union -> function-after',
        'union -> function-after -> function-plain',
        'union -> function-before',
        'union -> function-before -> function-plain',
        'union -> function-wrap',
        'union -> function-wrap -> int',
        'union -> function-wrap -> function-plain',
        'union -> function-plain',
        'union -> function-plain -> function-plain',
        'union -> default',
        'union -> default -> int',
        'union -> default -> function-plain',
        'union -> nullable',
        'union -> nullable -> int',
        'union -> nullable -> function-plain',
        'union -> union',
        'union -> union -> int',
        'union -> union -> str',
        'union -> union -> function-plain',
        'union -> tagged-union',
        'union -> tagged-union -> int',
        'union -> tagged-union -> str',
        'union -> tagged-union -> function-plain',
        'union -> chain',
        'union -> chain -> int',
        'union -> chain -> str',
        'union -> chain -> function-plain',
        'union -> lax-or-strict',
        'union -> lax-or-strict -> int',
        'union -> lax-or-strict -> str',
        'union -> lax-or-strict -> function-plain',
        'union -> json-or-python',
        'union -> json-or-python -> int',
        'union -> json-or-python -> str',
        'union -> json-or-python -> function-plain',
        'union -> typed-dict',
        'union -> typed-dict -> int',
        'union -> typed-dict -> int',
        'union -> typed-dict -> int -> int',
        'union -> typed-dict -> function-plain',
        'union -> model',
        'union -> dataclass',
        'union -> dataclass -> dataclass-args',
        'union -> dataclass -> dataclass-args -> int',
        'union -> call',
        'union -> call -> arguments',
        'union -> call -> arguments -> function-plain',
        'union -> call -> function-plain',
        'union -> custom-error',
        'union -> custom-error -> int',
        'union -> custom-error -> function-plain',
        'union -> json',
        'union -> json -> int',
        'union -> json -> function-plain',
        'union -> url',
        'union -> url -> function-plain',
        'union -> multi-host-url',
        'union -> multi-host-url -> function-plain',
        'union -> definitions',
        'union -> definitions -> int',
        'union -> definitions -> int',
        'union -> definition-ref',
        'union -> definition-ref -> function-plain',
        'union -> uuid',
        'union -> uuid -> function-plain',
    ]


def test_filter_has_ref() -> None:
    handler = TrackingHandler()

    walk = WalkCoreSchema(
        visit_core_schema=WalkCoreSchemaFilterBuilder.has_ref().build(handler),
        visit_ser_schema=WalkCoreSchemaFilterBuilder.has_ref().build(handler),
    )

    schema = cs.chain_schema(
        [
            cs.int_schema(ref='int'),
            cs.str_schema(
                serialization=cs.wrap_serializer_function_ser_schema(
                    wrap_ser_func,
                    schema=cs.str_schema(ref='str'),
                ),
            ),
            cs.list_schema(
                cs.float_schema(ref='float'),
            ),
        ]
    )

    walk.walk(schema)

    # insert_assert(handler.called)
    assert handler.called == ['int', 'str', 'float']


def test_filter_type() -> None:
    handler = TrackingHandler()

    walk = WalkCoreSchema(
        visit_core_schema=WalkCoreSchemaFilterBuilder.has_type('float').build(handler),
        visit_ser_schema=WalkCoreSchemaFilterBuilder.has_type('function-wrap').build(handler),
    )

    schema = cs.chain_schema(
        [
            cs.int_schema(),
            cs.str_schema(
                serialization=cs.wrap_serializer_function_ser_schema(
                    wrap_ser_func,
                    schema=cs.str_schema(),
                ),
            ),
            cs.list_schema(
                cs.float_schema(),
            ),
        ]
    )

    walk.walk(schema)

    # insert_assert(handler.called)
    assert handler.called == ['function-wrap', 'float']


def test_filter_and() -> None:
    handler = TrackingHandler()

    walk = WalkCoreSchema(
        visit_core_schema=(WalkCoreSchemaFilterBuilder.has_type('float') & WalkCoreSchemaFilterBuilder.has_ref()).build(
            handler
        ),
        visit_ser_schema=(
            WalkCoreSchemaFilterBuilder.has_type('function-wrap')
            & WalkCoreSchemaFilterBuilder.predicate(lambda s: s['schema']['type'] == 'str')
        ).build(handler),
    )

    schema = cs.chain_schema(
        [
            cs.int_schema(ref='int'),
            cs.str_schema(
                serialization=cs.wrap_serializer_function_ser_schema(
                    wrap_ser_func,
                    schema=cs.int_schema(),
                ),
            ),
            cs.str_schema(
                serialization=cs.wrap_serializer_function_ser_schema(
                    wrap_ser_func,
                    schema=cs.str_schema(),
                ),
            ),
            cs.str_schema(
                serialization=cs.plain_serializer_function_ser_schema(plain_ser_func),
            ),
            cs.float_schema(),
            cs.list_schema(
                cs.float_schema(),
            ),
            cs.list_schema(
                cs.float_schema(ref='float'),
            ),
        ]
    )

    walk.walk(schema)

    # insert_assert(handler.called)
    assert handler.called == ['function-wrap', 'float']
