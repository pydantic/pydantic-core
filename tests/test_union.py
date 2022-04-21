import pytest

from pydantic_core import SchemaValidator, ValidationError


def test_union():
    v = SchemaValidator({'type': 'union', 'choices': [{'type': 'bool'}, {'type': 'int'}]})
    assert v.validate_python('true') is True
    assert v.validate_python('123') == 123


class TestModelClass:
    class ModelA:
        pass

    class ModelB:
        pass

    v = SchemaValidator(
        {
            'type': 'union',
            'choices': [
                {
                    'type': 'model-class',
                    'class': ModelA,
                    'model': {'type': 'model', 'fields': {'a': {'type': 'int'}, 'b': {'type': 'str'}}},
                },
                {
                    'type': 'model-class',
                    'class': ModelB,
                    'model': {'type': 'model', 'fields': {'c': {'type': 'int'}, 'd': {'type': 'str'}}},
                },
            ],
        }
    )

    def test_model_a(self):
        m_a = self.v.validate_python({'a': 1, 'b': 'hello'})
        assert isinstance(m_a, self.ModelA)
        assert m_a.a == 1
        assert m_a.b == 'hello'

    def test_model_b(self):
        m_b = self.v.validate_python({'c': 2, 'd': 'again'})
        assert isinstance(m_b, self.ModelB)
        assert m_b.c == 2
        assert m_b.d == 'again'

    def test_exact_check(self):
        m_b = self.v.validate_python({'c': 2, 'd': 'again'})
        assert isinstance(m_b, self.ModelB)

        m_b2 = self.v.validate_python(m_b)
        assert m_b2 is m_b

    def test_error(self):
        with pytest.raises(ValidationError) as exc_info:
            self.v.validate_python({'a': 2})
        assert exc_info.value.errors() == [
            {
                'kind': 'missing',
                'loc': ['TestModelClass.ModelA', 'b'],
                'message': 'Field required',
                'input_value': {'a': 2},
            },
            {
                'kind': 'missing',
                'loc': ['TestModelClass.ModelB', 'c'],
                'message': 'Field required',
                'input_value': {'a': 2},
            },
            {
                'kind': 'missing',
                'loc': ['TestModelClass.ModelB', 'd'],
                'message': 'Field required',
                'input_value': {'a': 2},
            },
        ]


class TestModelClassSimilar:
    class ModelA:
        pass

    class ModelB:
        pass

    v = SchemaValidator(
        {
            'type': 'union',
            'choices': [
                {
                    'type': 'model-class',
                    'class': ModelA,
                    'model': {'type': 'model', 'fields': {'a': {'type': 'int'}, 'b': {'type': 'str'}}},
                },
                {
                    'type': 'model-class',
                    'class': ModelB,
                    'model': {
                        'type': 'model',
                        'fields': {'a': {'type': 'int'}, 'b': {'type': 'str'}, 'c': {'type': 'float', 'default': 1.0}},
                    },
                },
            ],
        }
    )

    def test_model_a(self):
        m_a = self.v.validate_python({'a': 1, 'b': 'hello'})
        assert isinstance(m_a, self.ModelA)
        assert m_a.a == 1
        assert m_a.b == 'hello'
        assert not hasattr(m_a, 'c')

    def test_model_b_ignored(self):
        # first choice works, so second choice is not used
        m_a = self.v.validate_python({'a': 1, 'b': 'hello', 'c': 2.0})
        assert isinstance(m_a, self.ModelA)
        assert m_a.a == 1
        assert m_a.b == 'hello'
        assert not hasattr(m_a, 'c')
