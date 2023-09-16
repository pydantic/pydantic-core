from typing import List

import pytest

from pydantic_core import PydanticUndefined, SchemaValidator, ValidationError, core_schema


def test_model_root():
    class RootModel:
        __slots__ = '__dict__', '__pydantic_fields_set__', '__pydantic_extra__', '__pydantic_private__'
        root: List[int]

    v = SchemaValidator(
        core_schema.model_schema(RootModel, core_schema.list_schema(core_schema.int_schema()), root_model=True)
    )
    assert repr(v).startswith('SchemaValidator(title="RootModel", validator=Model(\n')

    m = v.construct_python([1, 2, '3'])
    assert isinstance(m, RootModel)
    assert m.root == [1, 2, '3']
    assert m.__dict__ == {'root': [1, 2, '3']}

    m = v.construct_json('[1, 2, "3"]')
    assert isinstance(m, RootModel)
    assert m.root == [1, 2, '3']
    assert m.__dict__ == {'root': [1, 2, '3']}

    # Doesn't fail because construct
    m = v.construct_python('wrong')
    assert isinstance(m, RootModel)
    assert m.root == 'wrong'
    assert m.__dict__ == {'root': 'wrong'}


def test_reconstruct():
    class RootModel:
        __slots__ = '__dict__', '__pydantic_fields_set__', '__pydantic_extra__', '__pydantic_private__'
        root: List[int]

    v = SchemaValidator(
        core_schema.model_schema(
            RootModel,
            core_schema.list_schema(core_schema.int_schema()),
            root_model=True,
            reconstruct_instances='always',
        )
    )
    m = RootModel()
    m = v.construct_python([1, '2'])
    assert isinstance(m, RootModel)
    assert m.root == [1, '2']
    assert m.__pydantic_fields_set__ == {'root'}

    m2 = v.construct_python(m)
    assert m2 is not m
    assert isinstance(m2, RootModel)
    assert m2.root == [1, '2']
    assert m.__pydantic_fields_set__ == {'root'}


def test_reconstruct_with_default():
    class RootModel:
        __slots__ = '__dict__', '__pydantic_fields_set__', '__pydantic_extra__', '__pydantic_private__'
        root: int = 42

    v = SchemaValidator(
        core_schema.model_schema(
            RootModel,
            core_schema.with_default_schema(core_schema.int_schema(), default=42),
            root_model=True,
            reconstruct_instances='always',
        )
    )
    m = RootModel()
    m = v.construct_python(PydanticUndefined)
    assert isinstance(m, RootModel)
    assert m.root == 42
    assert m.__pydantic_fields_set__ == set()

    m2 = v.construct_python(m)
    assert m2 is not m
    assert isinstance(m2, RootModel)
    assert m2.root == 42
    assert m.__pydantic_fields_set__ == set()


def test_init():
    class RootModel:
        __slots__ = '__dict__', '__pydantic_fields_set__', '__pydantic_extra__', '__pydantic_private__'
        root: str

    v = SchemaValidator(
        core_schema.model_schema(RootModel, core_schema.str_schema(), root_model=True, revalidate_instances='always')
    )

    ans = v.construct_python('foobar')
    assert ans.root == 'foobar'


def test_assignment():
    class RootModel:
        __slots__ = '__dict__', '__pydantic_fields_set__', '__pydantic_extra__', '__pydantic_private__'
        root: str

    v = SchemaValidator(core_schema.model_schema(RootModel, core_schema.str_schema(), root_model=True))

    m = v.construct_python('foobar')
    assert m.root == 'foobar'

    m2 = v.validate_assignment(m, 'root', 'baz')
    assert m2 is m
    assert m.root == 'baz'

    with pytest.raises(ValidationError) as exc_info:
        v.validate_assignment(m, 'different', 'baz')

    # insert_assert(exc_info.value.errors(include_url=False))
    assert exc_info.value.errors(include_url=False) == [
        {
            'type': 'no_such_attribute',
            'loc': ('different',),
            'msg': "Object has no attribute 'different'",
            'input': 'baz',
            'ctx': {'attribute': 'different'},
        }
    ]


def test_field_function():
    call_infos = []

    class RootModel:
        __slots__ = '__dict__', '__pydantic_fields_set__', '__pydantic_extra__', '__pydantic_private__'
        root: str

    def f(input_value: str, info):
        call_infos.append(repr(info))
        return input_value + ' validated'

    v = SchemaValidator(
        core_schema.model_schema(
            RootModel, core_schema.field_after_validator_function(f, 'root', core_schema.str_schema()), root_model=True
        )
    )
    # Does not call validator
    m = v.construct_python('foobar')
    assert isinstance(m, RootModel)
    assert m.root == 'foobar'
    assert call_infos == []

    # Does call validator
    m2 = v.validate_assignment(m, 'root', 'baz', context='assignment call')
    assert m2 is m
    assert m.root == 'baz validated'
    assert call_infos == ["FieldValidationInfo(config=None, context='assignment call', field_name='root')"]


def test_extra():
    class RootModel:
        __slots__ = '__dict__', '__pydantic_fields_set__', '__pydantic_extra__', '__pydantic_private__'
        root: int

    v = SchemaValidator(core_schema.model_schema(RootModel, core_schema.int_schema(), root_model=True))

    m = v.construct_python(1)

    with pytest.raises(AttributeError):
        m.__pydantic_extra__


@pytest.mark.parametrize('recursive', [False, True])
def test_fields_set(recursive: bool):
    assert core_schema.PydanticUndefined is PydanticUndefined

    class RootModel:
        __slots__ = '__dict__', '__pydantic_fields_set__', '__pydantic_extra__', '__pydantic_private__'
        root: int = 42

    v = SchemaValidator(
        core_schema.model_schema(
            RootModel, core_schema.with_default_schema(core_schema.int_schema(), default=42), root_model=True
        )
    )

    m: RootModel = v.construct_python(1, recursive=recursive)
    assert m.root == 1
    assert m.__pydantic_fields_set__ == {'root'}

    m: RootModel = v.construct_python(PydanticUndefined, recursive=recursive)
    assert m.root == 42
    assert m.__pydantic_fields_set__ == set()


def test_construct_from_validate_default():
    class RootModel:
        __slots__ = '__dict__', '__pydantic_fields_set__', '__pydantic_extra__', '__pydantic_private__'
        root: int

    class Model:
        __slots__ = '__dict__', '__pydantic_fields_set__', '__pydantic_extra__', '__pydantic_private__'
        value: RootModel = 42

    v = SchemaValidator(
        core_schema.model_schema(
            Model,
            core_schema.model_fields_schema(
                {
                    'value': core_schema.model_field(
                        core_schema.with_default_schema(
                            core_schema.model_schema(RootModel, core_schema.int_schema(), root_model=True),
                            default=42,
                            construct_default=True,
                        )
                    )
                }
            ),
        )
    )

    # Construct new model
    nm = v.construct_python({}, recursive=True)

    assert nm.value.root == 42
    assert nm.value.__pydantic_fields_set__ == {'root'}


def test_model_root_recursive():
    class Model:
        __slots__ = '__dict__', '__pydantic_fields_set__', '__pydantic_extra__', '__pydantic_private__'
        a: int
        b: int

        def __init__(self, a, b):
            self.a = a
            self.b = b

        def __eq__(self, other):
            return self.a == other.a and self.b == other.b

    class RootModel:
        __slots__ = '__dict__', '__pydantic_fields_set__', '__pydantic_extra__', '__pydantic_private__'
        root: List[Model]

    v = SchemaValidator(
        core_schema.model_schema(
            RootModel,
            core_schema.list_schema(
                items_schema=core_schema.model_schema(
                    Model,
                    core_schema.model_fields_schema(
                        {
                            'a': core_schema.model_field(core_schema.int_schema()),
                            'b': core_schema.model_field(core_schema.int_schema()),
                        }
                    ),
                )
            ),
            root_model=True,
        )
    )

    m = v.construct_python(None)
    assert isinstance(m, RootModel)
    assert m.root is None
    m = v.construct_python(None, recursive=True)
    assert isinstance(m, RootModel)
    assert m.root is None

    m = v.construct_python(['wrong', {'a': 10, 'b': 'wrong'}])
    assert isinstance(m, RootModel)
    assert m.root == ['wrong', {'a': 10, 'b': 'wrong'}]
    m = v.construct_python(['wrong', {'a': 10, 'b': 'wrong'}], recursive=True)
    assert isinstance(m, RootModel)
    assert m.root == ['wrong', Model(10, 'wrong')]
