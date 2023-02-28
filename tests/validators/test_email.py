import random
import string
from typing import Optional, Union

import pytest

from pydantic_core import Email, SchemaValidator, ValidationError, core_schema

from ..conftest import Err, PyAndJson, plain_repr


def test_email_ok(py_and_json: PyAndJson):
    v = py_and_json(core_schema.email_schema())
    email = v.validate_test('john.doe@example.com')

    assert isinstance(email, Email)
    assert str(email) == 'john.doe@example.com'
    assert repr(email) == "Email('john.doe@example.com')"
    assert email.domain == 'example.com'
    assert email.local_part == 'john.doe'


def test_email_from_constructor_ok():
    email = Email('john.doe@example.com')

    assert isinstance(email, Email)
    assert str(email) == 'john.doe@example.com'
    assert repr(email) == "Email('john.doe@example.com')"
    assert email.domain == 'example.com'
    assert email.local_part == 'john.doe'


def test_email_repr():
    v = SchemaValidator({'type': 'email'})
    assert plain_repr(v) == 'SchemaValidator(name="email",validator=Email(EmailValidator{strict:false}),slots=[])'


@pytest.fixture(scope='module', name='email_validator')
def email_validator_fixture():
    return SchemaValidator(core_schema.email_schema())


SCHEMA_VALIDATOR_MODE = 'SCHEMA_VALIDATOR'
EMAIL_CLASS_MODE = 'EMAIL_CLASS'


def email_test_case_helper(
    email: str, expected: Union[Err, str], validator_mode: str, email_validator: Optional[SchemaValidator] = None
):
    if isinstance(expected, Err):
        with pytest.raises(ValidationError) as exc_info:
            if validator_mode == SCHEMA_VALIDATOR_MODE:
                email_validator.validate_python(email)
            elif validator_mode == EMAIL_CLASS_MODE:
                Email(email)
            else:
                raise ValueError(f'Unknown validator mode: {validator_mode}')
        assert exc_info.value.error_count() == 1
        error = exc_info.value.errors()[0]
        assert error['type'] == 'email_parsing'
        assert error['ctx']['error'] == expected.message
    else:
        if validator_mode == SCHEMA_VALIDATOR_MODE:
            output_email = email_validator.validate_python(email)
        elif validator_mode == EMAIL_CLASS_MODE:
            output_email = Email(email)
        else:
            raise ValueError(f'Unknown validator mode: {validator_mode}')
        assert isinstance(output_email, (Email,))
        if isinstance(expected, str):
            assert str(output_email) == expected
        else:
            assert isinstance(expected, dict)
            output_parts = {}
            for key in expected:
                if key == 'str()':
                    output_parts[key] = str(output_email)
                elif key.endswith('()'):
                    output_parts[key] = getattr(output_email, key[:-2])()
                else:
                    output_parts[key] = getattr(output_email, key)
            assert output_parts == expected


def generate_random_length_str(n: int) -> str:
    return ''.join(random.choice(string.ascii_uppercase + string.digits) for _ in range(n))


@pytest.mark.parametrize('mode', [SCHEMA_VALIDATOR_MODE, EMAIL_CLASS_MODE])
@pytest.mark.parametrize(
    'email,expected',
    [
        ('simple@example.com', {'str()': 'simple@example.com', 'domain': 'example.com', 'local_part': 'simple'}),
        # Failing, email_address doesnt support this
        (
            's <simple@example.com>',
            {'str()': 's <simple@example.com>', 'domain': 'example.com', 'local_part': 'simple'},
        ),
        (
            'very.common@example.com',
            {'str()': 'very.common@example.com', 'domain': 'example.com', 'local_part': 'very.common'},
        ),
        (
            'disposable.style.email.with+symbol@example.com',
            {
                'str()': 'disposable.style.email.with+symbol@example.com',
                'domain': 'example.com',
                'local_part': 'disposable.style.email.with+symbol',
            },
        ),
        (  # may go to user.name@example.com inbox depending on mail server
            'user.name+tag+sorting@example.com',
            {
                'str()': 'user.name+tag+sorting@example.com',
                'domain': 'example.com',
                'local_part': 'user.name+tag+sorting',
            },
        ),
        (  # one-letter local-part
            'x@example.com',
            {'str()': 'x@example.com', 'domain': 'example.com', 'local_part': 'x'},
        ),
        (
            'example-indeed@strange-example.com',
            {
                'str()': 'example-indeed@strange-example.com',
                'domain': 'strange-example.com',
                'local_part': 'example-indeed',
            },
        ),
        (  # local domain name with no TLD, although ICANN highly discourages dotless email addresses
            'admin@mailserver1',
            {'str()': 'admin@mailserver1', 'domain': 'mailserver1', 'local_part': 'admin'},
        ),
        (  # space between the quotes
            '" "@example.org',
            {'str()': '" "@example.org', 'domain': 'example.org', 'local_part': '" "'},
        ),
        (  # quoted double dot
            '"john..doe"@example.org',
            {'str()': '"john..doe"@example.org', 'domain': 'example.org', 'local_part': '"john..doe"'},
        ),
        (  # bangified host route used for uucp mailers
            'mailhost!username@example.org',
            {'str()': 'mailhost!username@example.org', 'domain': 'example.org', 'local_part': 'mailhost!username'},
        ),
        (  # % escaped mail route to user@example.com via example.org
            'user%example.com@example.org',
            {'str()': 'user%example.com@example.org', 'domain': 'example.org', 'local_part': 'user%example.com'},
        ),
        ('jsmith@[192.168.2.1]', {'str()': 'jsmith@[192.168.2.1]', 'domain': '[192.168.2.1]', 'local_part': 'jsmith'}),
        (
            'jsmith@[IPv6:2001:db8::1]',
            {'str()': 'jsmith@[IPv6:2001:db8::1]', 'domain': '[IPv6:2001:db8::1]', 'local_part': 'jsmith'},
        ),
        (
            'user+mailbox/department=shipping@example.com',
            {
                'str()': 'user+mailbox/department=shipping@example.com',
                'domain': 'example.com',
                'local_part': 'user+mailbox/department=shipping',
            },
        ),
        ('用户@例子.广告', {'str()': '用户@例子.广告', 'domain': '例子.广告', 'local_part': '用户'}),  # Chinese
        # ( # Hindi -> TODO: Not working... assuming something to do with encoding between python/rust
        #     'अजय@डाटा.भारत',
        #     {
        #         'str()': 'अजय@डाटा.भारत',
        #         'domain': 'अजय',
        #         'local_part': 'डाटा.भारत',
        #     },
        # ),
        (  # Ukranian
            'квіточка@пошта.укр',
            {'str()': 'квіточка@пошта.укр', 'domain': 'пошта.укр', 'local_part': 'квіточка'},
        ),
        ('θσερ@εχαμπλε.ψομ', {'str()': 'θσερ@εχαμπλε.ψομ', 'domain': 'εχαμπλε.ψομ', 'local_part': 'θσερ'}),  # Greek
        (  # German
            'Dörte@Sörensen.example.com',
            {'str()': 'Dörte@Sörensen.example.com', 'domain': 'Sörensen.example.com', 'local_part': 'Dörte'},
        ),
        ('коля@пример.рф', {'str()': 'коля@пример.рф', 'domain': 'пример.рф', 'local_part': 'коля'}),  # Russian
        ## Errs
        ('john.doe', Err("Missing separator character '@'.")),
        ('@example.com', Err('Local part is empty.')),
        (' @example.com', Err('Invalid character.')),
        ('john.doe@.com', Err('Invalid character.')),
        (f'{generate_random_length_str(64)}@example.com', {'domain': 'example.com'}),
        (f'{generate_random_length_str(65)}@example.com', Err('Local part is too long. Length limit: 64')),
        ('john.doe@', Err('Domain is empty.')),
        (f'john.doe@{generate_random_length_str(63)}.example.com', {'local_part': 'john.doe'}),
        (f'john.doe@{generate_random_length_str(64)}.example.com', Err('A sub-domain is too long. Length limit: 63')),
        (f'john.doe@example.{generate_random_length_str(246)}', Err('A sub-domain is too long. Length limit: 63')),
        (f'john.doe@example.{generate_random_length_str(247)}', Err('Domain is too long. Length limit: 254')),
        (f'john.doe@example.{generate_random_length_str(254)}', Err('Domain is too long. Length limit: 254')),
        # Error available but not used in email_address rust package
        # ('john.doe@com', Err('Too few parts in the domain')),
        # ('@example.com', Err('Invalid placement of the domain separator')),
        # ('@example.com', Err('Invalid IP Address specified for domain.')),
        # ('@example.com', Err('Quotes around the local-part are unbalanced.')),
        # ('@example.com', Err('A comment was badly formed.')),
    ],
)
def test_email_cases(email_validator, email, expected, mode):
    email_test_case_helper(email, expected, mode, email_validator)


@pytest.fixture(scope='module', name='strict_email_validator')
def strict_email_validator_fixture():
    return SchemaValidator(core_schema.email_schema(), {'strict': True})


def test_wrong_type_lax(email_validator):
    assert str(email_validator.validate_python('simple@example.com')) == 'simple@example.com'
    assert str(email_validator.validate_python(b'simple@example.com')) == 'simple@example.com'
    with pytest.raises(ValidationError, match=r'Input should be a string or email address \[type=email_type,'):
        email_validator.validate_python(123)

    # runtime strict
    with pytest.raises(ValidationError, match=r'Input should be a string or email address \[type=email_type,'):
        email_validator.validate_python(b'simple@example.com', strict=True)


def test_wrong_type_strict(strict_email_validator):
    email = strict_email_validator.validate_python('simple@example.com')
    assert str(email) == 'simple@example.com'
    assert str(strict_email_validator.validate_python(email)) == 'simple@example.com'
    with pytest.raises(ValidationError, match=r'Input should be a string or email address \[type=email_type,'):
        strict_email_validator.validate_python(b'simple@example.com')
    with pytest.raises(ValidationError, match=r'Input should be a string or email address \[type=email_type,'):
        strict_email_validator.validate_python(123)
