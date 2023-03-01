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
    return ''.join(random.choice(string.ascii_letters) for _ in range(n))


@pytest.mark.parametrize('mode', [SCHEMA_VALIDATOR_MODE, EMAIL_CLASS_MODE])
@pytest.mark.parametrize(
    'email,expected',
    [
        ('simple@example.com', {'str()': 'simple@example.com', 'domain': 'example.com', 'local_part': 'simple'}),
        (
            'simple@example.com',
            {
                'str()': 'simple@example.com',
                'domain': 'example.com',
                'local_part': 'simple',
                'name': '',
                'original_email': 'simple@example.com',
            },
        ),
        (
            's <simple@example.com>',
            {
                'str()': 's <simple@example.com>',
                'domain': 'example.com',
                'local_part': 'simple',
                'name': 's',
                'original_email': 's <simple@example.com>',
            },
        ),
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
        # (  # local domain name with no TLD, although ICANN highly discourages dotless email addresses
        #     'admin@mailserver1',
        #     {'str()': 'admin@mailserver1', 'domain': 'mailserver1', 'local_part': 'admin'},
        # ),
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
        (f'john.doe@example.{generate_random_length_str(63)}', {'local_part': 'john.doe'}),
        (f'john.doe@example.{generate_random_length_str(64)}', Err('A sub-domain is too long. Length limit: 63')),
        (
            f'john.doe@example.{generate_random_length_str(255-len("example."))}',
            Err('Domain is too long. Length limit: 254'),
        ),
        (f'john.doe@e.{generate_random_length_str(250)}', Err('A sub-domain is too long. Length limit: 63')),
        # Error available but not used in email_address rust package
        ('john.doe@com', Err('Too few parts in the domain')),
        # ('@example.com', Err('Invalid placement of the domain separator')),
        # ('@example.com', Err('Invalid IP Address specified for domain.')),
        # ('@example.com', Err('Quotes around the local-part are unbalanced.')),
        # ('@example.com', Err('A comment was badly formed.')),
        #
        # Tests derived from python-email-validator
        # Positive cases
        ('Abc@example.tld', {'local_part': 'Abc', 'domain': 'example.tld', 'email': 'Abc@example.tld'}),
        (
            'Abc.123@test-example.com',
            {'local_part': 'Abc.123', 'domain': 'test-example.com', 'email': 'Abc.123@test-example.com'},
        ),
        (
            'user+mailbox/department=shipping@example.tld',
            {
                'local_part': 'user+mailbox/department=shipping',
                'domain': 'example.tld',
                'email': 'user+mailbox/department=shipping@example.tld',
            },
        ),
        (
            "!#$%&'*+-/=?^_`.{|}~@example.tld",
            {
                'local_part': "!#$%&'*+-/=?^_`.{|}~",
                'domain': 'example.tld',
                'email': "!#$%&'*+-/=?^_`.{|}~@example.tld",
            },
        ),
        ('伊昭傑@郵件.商務', {'local_part': '伊昭傑', 'domain': '郵件.商務', 'email': '伊昭傑@郵件.商務'}),
        ('राम@मोहन.ईन्फो', {'local_part': 'राम', 'domain': 'मोहन.ईन्फो', 'email': 'राम@मोहन.ईन्फो'}),
        ('юзер@екзампл.ком', {'local_part': 'юзер', 'domain': 'екзампл.ком', 'email': 'юзер@екзампл.ком'}),
        ('θσερ@εχαμπλε.ψομ', {'local_part': 'θσερ', 'domain': 'εχαμπλε.ψομ', 'email': 'θσερ@εχαμπλε.ψομ'}),
        ('葉士豪@臺網中心.tw', {'local_part': '葉士豪', 'domain': '臺網中心.tw', 'email': '葉士豪@臺網中心.tw'}),
        ('jeff@臺網中心.tw', {'local_part': 'jeff', 'domain': '臺網中心.tw', 'email': 'jeff@臺網中心.tw'}),
        ('葉士豪@臺網中心.台灣', {'local_part': '葉士豪', 'domain': '臺網中心.台灣', 'email': '葉士豪@臺網中心.台灣'}),
        ('jeff葉@臺網中心.tw', {'local_part': 'jeff葉', 'domain': '臺網中心.tw', 'email': 'jeff葉@臺網中心.tw'}),
        ('ñoñó@example.tld', {'local_part': 'ñoñó', 'domain': 'example.tld', 'email': 'ñoñó@example.tld'}),
        ('我買@example.tld', {'local_part': '我買', 'domain': 'example.tld', 'email': '我買@example.tld'}),
        ('甲斐黒川日本@example.tld', {'local_part': '甲斐黒川日本', 'domain': 'example.tld', 'email': '甲斐黒川日本@example.tld'}),
        (
            'чебурашкаящик-с-апельсинами.рф@example.tld',
            {
                'local_part': 'чебурашкаящик-с-апельсинами.рф',
                'domain': 'example.tld',
                'email': 'чебурашкаящик-с-апельсинами.рф@example.tld',
            },
        ),
        # TODO: Unsure why is this broken
        # ( # Hindi
        #     'उदाहरण.परीक्ष@"domain".with.idn.tld',
        #     {
        #         'local_part': 'उदाहरण.परीक्ष',
        #         'domain': '"domain".with.idn.tld',
        #         'email': 'उदाहरण.परीक्ष@"domain".with.idn.tld',
        #     },
        # ),
        ('ιωάννης@εεττ.gr', {'local_part': 'ιωάννης', 'domain': 'εεττ.gr', 'email': 'ιωάννης@εεττ.gr'}),
        ## Negative cases
        ('white space@test', Err('Invalid character.')),
        ('\n@test', Err('Invalid character.')),
        ## TODO: Cannot find any reference in RFC's to these "invalid characters" assuming idna.uts46_remap will fix
        # ('\u2005@test', Err('Invalid character.')),  # four-per-em space (Zs)
        # ('\u009C@test', Err('Invalid character.')),  # string terminator (Cc)
        # ('\u200B@test', Err('Invalid character.')),  # zero-width space (Cf)
        # ('\u202Dforward-\u202Ereversed@test', Err('Invalid character.')),  # BIDI (Cf)
        # ('\uD800@test', Err('Invalid character.')),  # surrogate (Cs)
        # ('\uE000@test', Err('Invalid character.')),  # private use (Co)
        # ('\uFDEF@test', Err('Invalid character.')),  # unassigned (Cn)
        # ('\u0300@test', Err('Invalid character.')),  # grave accent (M)
        ## TODO: These domains are currently explicitly rejected # Restricted domains flag?
        # ('me@anything.arpa', Err('Invalid character.')),
        # ('me@valid.invalid', Err('Invalid character.')),
        # ('me@link.local', Err('Invalid character.')),
        # ('me@host.localhost', Err('Invalid character.')),
        # ('me@onion.onion.onion', Err('Invalid character.')),
        # ('me@test.test.test', Err('Invalid character.')),
        ##
        # TODO: This is a valid test case, but expected
        # "The part after the @-sign is not valid. It should have a period." globally_deliverable flag
        ('my@localhost', Err('Too few parts in the domain')),
        ('my@.leadingdot.com', Err('Invalid character.')),
        ('my@twodots..com', Err('Invalid character.')),
        ('my@trailingdot.com.', Err('Invalid character.')),
        ('me@-leadingdash', Err('Invalid character.')),
        ('me@trailingdash-', Err('Invalid character.')),
        # TODO: idna.uts46_remap
        # ('my@．leadingfwdot.com', Err('Invalid character.')),
        # ('my@twofwdots．．.com', Err('Invalid character.')),
        # ('my@trailingfwdot.com．', Err('Invalid character.')),
        # ('me@－leadingdashfw', Err('Invalid character.')),
        # ('me@trailingdashfw－', Err('Invalid character.')),
        ('my@baddash.-.com', Err('Invalid character.')),
        ('my@baddash.-a.com', Err('Invalid character.')),
        ('my@baddash.b-.com', Err('Invalid character.')),
        # TODO: idna.uts46_remap
        # ('my@baddashfw.－.com', Err('Invalid character.')),
        # ('my@baddashfw.－a.com', Err('Invalid character.')),
        # ('my@baddashfw.b－.com', Err('Invalid character.')),
        ('my@example.com\n', Err('Invalid character.')),
        ('my@example\n.com', Err('Invalid character.')),
        ('.leadingdot@domain.com', Err('Invalid character.')),
        ('..twodots@domain.com', Err('Invalid character.')),
        ('twodots..here@domain.com', Err('Invalid character.')),
        # ('me@⒈wouldbeinvalid.com', Err('Invalid character.')), # TODO: No reference to RFC
        ('@example.com', Err('Local part is empty.')),
        ('\nmy@example.com', Err('Invalid character.')),
        ('m\ny@example.com', Err('Invalid character.')),
        ('my\n@example.com', Err('Invalid character.')),
        (
            '11111111112222222222333333333344444444445555555555666666666677777@example.com',
            Err('Local part is too long. Length limit: 64'),
        ),
        (
            '111111111122222222223333333333444444444455555555556666666666777777@example.com',
            Err('Local part is too long. Length limit: 64'),
        ),
        (
            'me@1111111111222222222233333333334444444444555555555.6666666666777777777788888888889999999999000000000.1111111111222222222233333333334444444444555555555.6666666666777777777788888888889999999999000000000.111111111122222222223333333333444444444455555555556.com',
            Err('Domain is too long. Length limit: 254'),
        ),
        (
            'me@1111111111222222222233333333334444444444555555555.6666666666777777777788888888889999999999000000000.1111111111222222222233333333334444444444555555555.6666666666777777777788888888889999999999000000000.1111111111222222222233333333334444444444555555555566.com',
            Err('Domain is too long. Length limit: 254'),
        ),
        (
            'me@中1111111111222222222233333333334444444444555555555.6666666666777777777788888888889999999999000000000.1111111111222222222233333333334444444444555555555.6666666666777777777788888888889999999999000000000.1111111111222222222233333333334444444444555555555566.com',
            Err('Domain is too long. Length limit: 254'),
        ),
        # TODO: These seem to be valid... - len(email.encode(idna) > 255)
        (
            'my.long.address@1111111111222222222233333333334444444444555555555.6666666666777777777788888888889999999999000000000.1111111111222222222233333333334444444444555555555.6666666666777777777788888888889999999999000000000.11111111112222222222333333333344444.info',
            Err('The Address is too long. Length limit: 254'),
        ),
        # ('my.long.address@λ111111111222222222233333333334444444444555555555.6666666666777777777788888888889999999999000000000.1111111111222222222233333333334444444444555555555.6666666666777777777788888888889999999999000000000.11111111112222222222333333.info', Err('The Address is too long. Length limit: 254')),  # noqa: E501
        (
            'my.long.address@λ111111111222222222233333333334444444444555555555.6666666666777777777788888888889999999999000000000.1111111111222222222233333333334444444444555555555.6666666666777777777788888888889999999999000000000.1111111111222222222233333333334444.info',
            Err('The Address is too long. Length limit: 254'),
        ),
        # ('my.λong.address@1111111111222222222233333333334444444444555555555.6666666666777777777788888888889999999999000000000.1111111111222222222233333333334444444444555555555.6666666666777777777788888888889999999999000000000.111111111122222222223333333333444.info', Err('The Address is too long. Length limit: 254')),  # noqa: E501
        (
            'my.λong.address@1111111111222222222233333333334444444444555555555.6666666666777777777788888888889999999999000000000.1111111111222222222233333333334444444444555555555.6666666666777777777788888888889999999999000000000.1111111111222222222233333333334444.info',
            Err('The Address is too long. Length limit: 254'),
        ),
        ('me@bad-tld-1', Err('Invalid character.')),
        ('me@bad.tld-2', Err('Invalid character.')),  # => TLD should? end with a letter
        (
            'me@x!',
            Err('Invalid character.'),
        ),  # A "name" (Net, Host, Gateway, or Domain name) is a text string up to 24 characters drawn from the alphabet (A-Z), digits (0-9), minus sign (-), and period (.).   # noqa: E501
        # ('me@xn--0.tld', Err('Invalid character.')), # => Cannot be decoded by punnycode
        # Labels within the class of R-LDH labels that are not prefixed with "xn--" are also not valid IDNA labels.  To allow for future use of mechanisms similar to IDNA, those labels MUST NOT be processed   # noqa: E501
        ('me@yy--0.tld', Err('Invalid label.')),
        # ('me@yy－－0.tld', Err('Invalid character.')),
        # ^1        The labels must follow the rules for ARPANET host names.  They must
        # start with a letter, end with a letter or digit, and have as interior
        # characters only letters, digits, and hyphen.  There are also some
        # restrictions on the length.  Labels must be 63 characters or less.
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
