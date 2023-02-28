import pytest

from pydantic_core import Email, SchemaSerializer, SchemaValidator, core_schema


def test_email():
    v = SchemaValidator(core_schema.email_schema())
    s = SchemaSerializer(core_schema.email_schema())

    email = v.validate_python('john.doe@example.com')
    assert isinstance(email, Email)
    assert str(email) == 'john.doe@example.com'
    assert email.domain == 'example.com'
    assert email.local_part == 'john.doe'

    assert s.to_python(email) == email
    assert s.to_python(email, mode='json') == 'john.doe@example.com'
    assert s.to_json(email) == b'"john.doe@example.com"'

    with pytest.warns(UserWarning, match='Expected `email` but got `str` - serialized value may not be as expected'):
        assert s.to_python('john.doe@example.com', mode='json') == 'john.doe@example.com'


def test_email_dict_keys():
    v = SchemaValidator(core_schema.email_schema())

    s = SchemaSerializer(core_schema.dict_schema(core_schema.email_schema()))
    email = v.validate_python('john.doe@example.com')
    assert s.to_python({email: 'foo'}) == {email: 'foo'}
    assert s.to_python({email: 'foo'}, mode='json') == {'john.doe@example.com': 'foo'}
    assert s.to_json({email: 'foo'}) == b'{"john.doe@example.com":"foo"}'


def test_any():
    email = Email('john.doe@example.com')

    s = SchemaSerializer(core_schema.any_schema())
    assert s.to_python(email) == email
    assert type(s.to_python(email)) == Email
