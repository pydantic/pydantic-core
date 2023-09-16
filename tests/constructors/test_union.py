import pytest

from pydantic_core import SchemaValidator, core_schema


@pytest.mark.parametrize(
    'input_value,expected_value',
    [
        (True, True),
        (False, False),
        ('true', 'true'),
        ('false', 'false'),
        (1, 1),
        (0, 0),
        (123, 123),
        ('123', '123'),
        ('0', '0'),
        ('1', '1'),
        (b'hello', b'hello'),  # no normalization is done
    ],
)
def test_union_int_bool(input_value, expected_value):
    """Primitive types aren't affected my annotations when constructed"""
    v = SchemaValidator({'type': 'union', 'choices': [{'type': 'int'}, {'type': 'bool'}]})
    assert v.construct_python(input_value) == expected_value


def test_union_of_simple():
    choices = [core_schema.int_schema(), core_schema.float_schema()]

    # construction recursion is ignored for simple types, regardless of `mode`
    v = SchemaValidator(core_schema.union_schema(choices, mode='smart'))
    out = v.construct_python(1)
    assert out == 1
    assert isinstance(out, int)

    out = v.construct_python(1.0)
    assert out == 1.0
    assert isinstance(out, float)

    v = SchemaValidator(core_schema.union_schema(choices, mode='left_to_right'))
    out = v.construct_python(1)
    assert out == 1
    assert isinstance(out, int)

    out = v.construct_python(1.0)
    assert out == 1.0
    assert isinstance(out, float)


class TestModelClass:
    class ModelA:
        pass

    class ModelB:
        pass

    @pytest.fixture(scope='class')
    def schema_validator(self) -> SchemaValidator:
        return SchemaValidator(
            {
                'type': 'union',
                'choices': [
                    {
                        'type': 'model',
                        'cls': self.ModelA,
                        'schema': {
                            'type': 'model-fields',
                            'fields': {
                                'a': {'type': 'model-field', 'schema': {'type': 'int'}},
                                'b': {'type': 'model-field', 'schema': {'type': 'str'}},
                            },
                        },
                    },
                    {
                        'type': 'model',
                        'cls': self.ModelB,
                        'schema': {
                            'type': 'model-fields',
                            'fields': {
                                'c': {'type': 'model-field', 'schema': {'type': 'int'}},
                                'd': {'type': 'model-field', 'schema': {'type': 'str'}},
                            },
                        },
                    },
                ],
            }
        )

    def test_model_a(self, schema_validator: SchemaValidator):
        m_a = schema_validator.construct_python({'a': 1, 'b': 'hello'})
        assert isinstance(m_a, self.ModelA)
        assert m_a.a == 1
        assert m_a.b == 'hello'

    def test_model_b(self, schema_validator: SchemaValidator):
        m_b = schema_validator.construct_python({'c': 2, 'd': 'again'})
        assert isinstance(m_b, self.ModelB)
        assert m_b.c == 2
        assert m_b.d == 'again'

    def test_exact_check(self, schema_validator: SchemaValidator):
        m_b = schema_validator.construct_python({'c': 2, 'd': 'again'})
        assert isinstance(m_b, self.ModelB)

        m_b2 = schema_validator.construct_python(m_b)
        assert m_b2 is m_b

    def test_incorrect_values(self, schema_validator: SchemaValidator):
        """Should still be an instance of ModelB based on the fields present, not their values"""
        m_b = schema_validator.construct_python({'c': 'wrong', 'd': set()})
        assert isinstance(m_b, self.ModelB)

        m_b2 = schema_validator.construct_python(m_b)
        assert m_b2 is m_b


class TestModelClassSimilar:
    class ModelA:
        pass

    class ModelB:
        pass

    @pytest.fixture(scope='class')
    def schema_validator(self) -> SchemaValidator:
        return SchemaValidator(
            {
                'type': 'union',
                'choices': [
                    {
                        'type': 'model',
                        'cls': self.ModelA,
                        'schema': {
                            'type': 'model-fields',
                            'fields': {
                                'a': {'type': 'model-field', 'schema': {'type': 'int'}},
                                'b': {'type': 'model-field', 'schema': {'type': 'str'}},
                            },
                        },
                    },
                    {
                        'type': 'model',
                        'cls': self.ModelB,
                        'schema': {
                            'type': 'model-fields',
                            'fields': {
                                'a': {'type': 'model-field', 'schema': {'type': 'int'}},
                                'b': {'type': 'model-field', 'schema': {'type': 'str'}},
                                'c': {
                                    'type': 'model-field',
                                    'schema': {'type': 'default', 'schema': {'type': 'float'}, 'default': 1.0},
                                },
                            },
                        },
                    },
                ],
            }
        )

    def test_model_a(self, schema_validator: SchemaValidator):
        m = schema_validator.construct_python({'a': 1, 'b': 'hello'})
        assert isinstance(m, self.ModelA)
        assert m.a == 1
        assert m.b == 'hello'
        assert not hasattr(m, 'c')

    def test_model_b_ignored(self, schema_validator: SchemaValidator):
        # first choice works, so second choice is not used
        m = schema_validator.construct_python({'a': 1, 'b': 'hello', 'c': 2.0})
        assert isinstance(m, self.ModelA)
        assert m.a == 1
        assert m.b == 'hello'
        assert not hasattr(m, 'c')

    def test_model_b_not_ignored(self, schema_validator: SchemaValidator):
        m1 = self.ModelB()
        m1.a = 1
        m1.b = 'hello'
        m1.c = 2.0
        m2 = schema_validator.construct_python(m1)
        assert isinstance(m2, self.ModelB)
        assert m2.a == 1
        assert m2.b == 'hello'
        assert m2.c == 2.0
