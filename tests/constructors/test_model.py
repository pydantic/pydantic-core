import pytest

from pydantic_core import SchemaValidator, ValidationError, core_schema


def test_standard_model_construct():
    class MyModel:
        # this is not required, but it avoids `__pydantic_fields_set__` being included in `__dict__`
        __slots__ = '__dict__', '__pydantic_fields_set__', '__pydantic_extra__', '__pydantic_private__'
        field_a: str
        field_b: int

    v = SchemaValidator(
        core_schema.model_schema(
            MyModel,
            core_schema.model_fields_schema(
                {
                    'field_a': core_schema.model_field(core_schema.str_schema()),
                    'field_b': core_schema.model_field(core_schema.int_schema()),
                }
            ),
        )
    )

    m = v.construct_python({'field_a': 'test', 'field_b': 12})
    assert isinstance(m, MyModel)
    assert m.field_a == 'test'
    assert m.field_b == 12
    assert m.__pydantic_extra__ is None
    assert m.__pydantic_fields_set__ == {'field_a', 'field_b'}
    assert m.__dict__ == {'field_a': 'test', 'field_b': 12}

    m2 = v.construct_python({'field_a': 12, 'field_b': 'test'})
    assert isinstance(m2, MyModel)
    assert m2.field_a == 12
    assert m2.field_b == 'test'
    assert m2.__pydantic_extra__ is None
    assert m2.__pydantic_fields_set__ == {'field_a', 'field_b'}
    assert m2.__dict__ == {'field_a': 12, 'field_b': 'test'}


@pytest.mark.parametrize('extra_behavior,extra', [('allow', {'field_c': 'extra'}), ('ignore', None), ('forbid', None)])
def test_model_construct_extra(extra_behavior, extra):
    class MyModel:
        # this is not required, but it avoids `__pydantic_fields_set__` being included in `__dict__`
        __slots__ = '__dict__', '__pydantic_fields_set__', '__pydantic_extra__', '__pydantic_private__'
        field_a: str
        field_b: int

    v = SchemaValidator(
        core_schema.model_schema(
            MyModel,
            core_schema.model_fields_schema(
                {
                    'field_a': core_schema.model_field(core_schema.str_schema()),
                    'field_b': core_schema.model_field(core_schema.int_schema()),
                },
                extra_behavior=extra_behavior,
            ),
        )
    )
    m = v.construct_python({'field_a': 'test', 'field_b': 12, 'field_c': 'extra'})
    assert isinstance(m, MyModel)
    assert m.field_a == 'test'
    assert m.field_b == 12
    assert m.__pydantic_extra__ == extra
    assert m.__pydantic_fields_set__ == {'field_a', 'field_b'}


def test_post_init_internal_error():
    class MyModel:
        __slots__ = '__dict__', '__pydantic_fields_set__', '__pydantic_extra__', '__pydantic_private__'
        field_a: str

        def wrong_signature(self):
            pass

    v = SchemaValidator(
        {
            'type': 'model',
            'cls': MyModel,
            'post_init': 'wrong_signature',
            'schema': {
                'type': 'model-fields',
                'fields': {'field_a': {'type': 'model-field', 'schema': {'type': 'str'}}},
            },
        }
    )
    with pytest.raises(TypeError, match=r'wrong_signature\(\) takes 1 positional argument but 2 were given'):
        v.construct_python({'field_a': 'test'})


def test_post_init_mutate():
    """__post_init__ works when calling construct:"""

    class MyModel:
        __slots__ = '__dict__', '__pydantic_fields_set__', '__pydantic_extra__', '__pydantic_private__'
        field_a: str
        field_b: int

        def call_me_maybe(self, context, **kwargs):
            self.field_a *= 2
            self.__pydantic_fields_set__ = {'field_a'}

    v = SchemaValidator(
        {
            'type': 'model',
            'cls': MyModel,
            'post_init': 'call_me_maybe',
            'schema': {
                'type': 'model-fields',
                'fields': {
                    'field_a': {'type': 'model-field', 'schema': {'type': 'str'}},
                    'field_b': {'type': 'model-field', 'schema': {'type': 'int'}},
                },
            },
        }
    )
    m = v.construct_python({'field_a': 'test', 'field_b': 'wrong'})
    assert isinstance(m, MyModel)
    assert m.field_a == 'testtest'
    assert m.field_b == 'wrong'
    assert m.__pydantic_fields_set__ == {'field_a'}
    assert m.__dict__ == {'field_a': 'testtest', 'field_b': 'wrong'}


def test_construct_frozen():
    class MyModel:
        __slots__ = {'__dict__', '__pydantic_fields_set__', '__pydantic_extra__', '__pydantic_private__'}

    v = SchemaValidator(
        core_schema.model_schema(
            MyModel,
            core_schema.model_fields_schema({'f': core_schema.model_field(core_schema.str_schema())}),
            frozen=True,
        )
    )

    m = v.construct_python({'f': 'x'})
    assert m.f == 'x'

    with pytest.raises(ValidationError) as exc_info:
        v.validate_assignment(m, 'f', 'y')

    # insert_assert(exc_info.value.errors(include_url=False))
    assert exc_info.value.errors(include_url=False) == [
        {'type': 'frozen_instance', 'loc': (), 'msg': 'Instance is frozen', 'input': 'y'}
    ]


def test_model_construct_setattr():
    setattr_calls = []

    class MyModel:
        field_a: str

        def __setattr__(self, key, value):
            setattr_calls.append((key, value))
            # don't do anything

    m1 = MyModel()
    m1.foo = 'bar'
    assert not hasattr(m1, 'foo')
    assert setattr_calls == [('foo', 'bar')]
    setattr_calls.clear()

    v = SchemaValidator(
        {
            'type': 'model',
            'cls': MyModel,
            'schema': {
                'type': 'model-fields',
                'fields': {'field_a': {'type': 'model-field', 'schema': {'type': 'str'}}},
            },
        }
    )
    m = v.construct_python({'field_a': 'test'})
    assert isinstance(m, MyModel)
    assert m.field_a == 'test'
    assert m.__pydantic_fields_set__ == {'field_a'}
    assert setattr_calls == []


@pytest.mark.parametrize('recursive', [False, True])
def test_construct_fields_set(recursive):
    """Straight from the docs"""

    class User:
        id: int
        age: int
        name: str = 'John Doe'

    v = SchemaValidator(
        {
            'type': 'model',
            'cls': User,
            'schema': {
                'type': 'model-fields',
                'fields': {
                    'id': {'type': 'model-field', 'schema': {'type': 'int'}},
                    'age': {'type': 'model-field', 'schema': {'type': 'int'}},
                    'name': {
                        'type': 'model-field',
                        'schema': {'type': 'default', 'schema': {'type': 'str'}, 'default': 'John Doe'},
                    },
                },
            },
        }
    )

    m = v.construct_python({'id': 'something', 'age': 22}, recursive=recursive)
    assert isinstance(m, User)
    assert m.id == 'something'
    assert m.age == 22
    assert m.name == 'John Doe'
    assert m.__pydantic_fields_set__ == {'id', 'age'}

    # Simulate `model_dump`
    model_data = {'id': 'something', 'age': 22, 'name': 'John Doe'}
    old_fields = m.__pydantic_fields_set__

    m = v.construct_python(model_data, fields_set=old_fields, recursive=recursive)
    assert m.id == 'something'
    assert m.age == 22
    assert m.name == 'John Doe'
    assert m.__pydantic_fields_set__ == {'id', 'age'}


def test_model_init_not_called():
    """Model.__init__ should not be called when calling `construct_python`"""
    pass  # TODO


def test_standard_model_construct_recursive():
    class MyModel:
        # this is not required, but it avoids `__pydantic_fields_set__` being included in `__dict__`
        __slots__ = '__dict__', '__pydantic_fields_set__', '__pydantic_extra__', '__pydantic_private__'
        field_a: str
        field_b: int

    class AnotherModel:
        child_model: MyModel

    v = SchemaValidator(
        core_schema.model_schema(
            AnotherModel,
            core_schema.model_fields_schema(
                {
                    'child_model': core_schema.model_field(
                        core_schema.model_schema(
                            MyModel,
                            core_schema.model_fields_schema(
                                {
                                    'field_a': core_schema.model_field(core_schema.str_schema()),
                                    'field_b': core_schema.model_field(core_schema.int_schema()),
                                }
                            ),
                        )
                    )
                }
            ),
        )
    )

    m = v.construct_python({'child_model': {'field_a': 'test1', 'field_b': 'test2'}})
    assert isinstance(m, AnotherModel)
    assert isinstance(m.child_model, dict)
    assert m.child_model == {'field_a': 'test1', 'field_b': 'test2'}

    mr = v.construct_python({'child_model': {'field_a': 'test1', 'field_b': 'test2'}}, recursive=True)
    assert isinstance(mr, AnotherModel)
    assert isinstance(mr.child_model, MyModel)
    assert mr.child_model.__dict__ == {'field_a': 'test1', 'field_b': 'test2'}
    assert mr.child_model.field_a == 'test1'
    assert mr.child_model.field_b == 'test2'


def test_model_init_not_called_recursive():
    """Model.__init__ of all submodels should not be called when calling `construct_python`"""
    pass  # TODO


def test_construct_model_instance():
    class MyModel:
        # this is not required, but it avoids `__pydantic_fields_set__` being included in `__dict__`
        __slots__ = '__dict__', '__pydantic_fields_set__', '__pydantic_extra__', '__pydantic_private__'
        field_a: str
        field_b: int

    v = SchemaValidator(
        core_schema.model_schema(
            MyModel,
            core_schema.model_fields_schema(
                {
                    'field_a': core_schema.model_field(core_schema.str_schema()),
                    'field_b': core_schema.model_field(core_schema.int_schema()),
                }
            ),
        )
    )

    m = v.construct_python({'field_a': 'test', 'field_b': 12})
    assert isinstance(m, MyModel)
    assert m.field_a == 'test'
    assert m.field_b == 12

    m2 = v.construct_python(m)
    assert isinstance(m2, MyModel)
    assert m2.field_a == 'test'
    assert m2.field_b == 12


def test_nested_fields_set_correct():
    class Child:
        # this is not required, but it avoids `__pydantic_fields_set__` being included in `__dict__`
        __slots__ = '__dict__', '__pydantic_fields_set__', '__pydantic_extra__', '__pydantic_private__'
        a: int
        b: float

    class Parent:
        # this is not required, but it avoids `__pydantic_fields_set__` being included in `__dict__`
        __slots__ = '__dict__', '__pydantic_fields_set__', '__pydantic_extra__', '__pydantic_private__'
        child: Child
        c: str

    child_schema = core_schema.model_schema(
        Child,
        core_schema.model_fields_schema(
            {
                'a': core_schema.model_field(core_schema.int_schema()),
                'b': core_schema.model_field(core_schema.float_schema()),
            }
        ),
    )

    v = SchemaValidator(
        core_schema.model_schema(
            Parent,
            core_schema.model_fields_schema(
                {'child': core_schema.model_field(child_schema), 'c': core_schema.model_field(core_schema.str_schema())}
            ),
        )
    )

    m = v.construct_python({'child': {'a': 10, 'b': 'wrong'}, 'c': 123}, recursive=True)
    assert isinstance(m, Parent)
    assert isinstance(m.child, Child)
    assert m.__pydantic_fields_set__ == {'child', 'c'}
    assert m.child.__pydantic_fields_set__ == {'a', 'b'}


def test_n_nested_model():
    class Child:
        # this is not required, but it avoids `__pydantic_fields_set__` being included in `__dict__`
        __slots__ = '__dict__', '__pydantic_fields_set__', '__pydantic_extra__', '__pydantic_private__'
        a: int
        b: float

        def __init__(self, a, b):
            self.a = a
            self.b = b

        def __eq__(self, other):
            return self.a == other.a and self.b == other.b

    child_schema = core_schema.model_schema(
        Child,
        core_schema.model_fields_schema(
            {
                'a': core_schema.model_field(core_schema.int_schema()),
                'b': core_schema.model_field(core_schema.float_schema()),
            }
        ),
    )

    # List[List[Child]]
    v = SchemaValidator(core_schema.list_schema(core_schema.list_schema(child_schema)))

    assert v.construct_python([[{'a': 10, 'b': 'wrong'}]]) == [[{'a': 10, 'b': 'wrong'}]]
    assert v.construct_python([[{'a': 10, 'b': 'wrong'}]], recursive=True) == [[Child(10, 'wrong')]]
