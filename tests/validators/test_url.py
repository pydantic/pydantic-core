import pytest

from pydantic_core import SchemaValidator, Url, ValidationError, core_schema

from ..conftest import PyAndJson


def test_url_ok(py_and_json: PyAndJson):
    v = py_and_json(core_schema.url_schema())
    url: Url = v.validate_test('https://example.com/foo/bar?baz=qux#quux')
    assert isinstance(url, Url)
    assert str(url) == 'https://example.com/foo/bar?baz=qux#quux'
    assert repr(url) == "Url('https://example.com/foo/bar?baz=qux#quux')"
    assert url.scheme == 'https'
    assert url.host == 'example.com'
    assert url.path == '/foo/bar'
    assert url.query == 'baz=qux'
    assert url.query_params() == [('baz', 'qux')]
    assert url.fragment == 'quux'
    assert url.username is None
    assert url.password is None
    assert url.port is None
    assert url.host_type == 'domain'


@pytest.mark.parametrize(
    'url,error',
    [
        ('xxx', 'relative URL without a base'),
        ('http://', 'empty host'),
        ('https://xn---', 'invalid international domain name'),
        ('http://example.com:65536', 'invalid port number'),
        ('http://1...1', 'invalid IPv4 address'),
        ('https://[2001:0db8:85a3:0000:0000:8a2e:0370:7334[', 'invalid IPv6 address'),
        ('https://[', 'invalid IPv6 address'),
        ('https://example com', 'invalid domain character'),
        ('/more', 'relative URL without a base'),
    ],
)
def test_url_error(py_and_json: PyAndJson, url, error):
    v = py_and_json(core_schema.url_schema())

    with pytest.raises(ValidationError) as exc_info:
        v.validate_test(url)
    assert exc_info.value.error_count() == 1
    assert exc_info.value.errors()[0]['ctx']['error'] == error


@pytest.mark.parametrize(
    'input_value,expected,host_type',
    [
        ('http://example.com', 'http://example.com/', 'domain'),
        ('https://£££.com', 'https://xn--9aaa.com/', 'international_domain'),
        ('https://xn--9aaa.com/', 'https://xn--9aaa.com/', 'international_domain'),
        ('http://à.א̈.com', 'http://xn--0ca.xn--ssa73l.com/', 'international_domain'),
        ('ftp://127.0.0.1', 'ftp://127.0.0.1/', 'ipv4'),
        ('wss://1.1.1.1', 'wss://1.1.1.1/', 'ipv4'),
        ('ftp://[2001:0db8:85a3:0000:0000:8a2e:0370:7334]', 'ftp://[2001:db8:85a3::8a2e:370:7334]/', 'ipv6'),
        ('https:/more', 'https://more/', 'domain'),
        ('https:more', 'https://more/', 'domain'),
    ],
)
def test_hosts(input_value, expected, host_type):
    v = SchemaValidator(core_schema.url_schema())
    url: Url = v.validate_python(input_value)
    assert isinstance(url, Url)
    assert str(url) == expected
    assert url.host_type == host_type


def test_host_required():
    v = SchemaValidator(core_schema.url_schema(host_required=True))
    url = v.validate_python('http://example.com')
    assert url.host == 'example.com'
    with pytest.raises(ValidationError, match=r'URL host required \[type=url_host_required,'):
        v.validate_python('unix:/run/foo.socket')


def test_no_host():
    v = SchemaValidator(core_schema.url_schema())
    url = v.validate_python('data:text/plain,Stuff')
    assert str(url) == 'data:text/plain,Stuff'
    assert url.host is None
    assert url.scheme == 'data'
    assert url.path == 'text/plain,Stuff'


def test_allowed_schemes_ok():
    v = SchemaValidator(core_schema.url_schema(allowed_schemes=['http', 'https']))
    url = v.validate_python(' https://example.com ')
    assert url.host == 'example.com'
    assert url.scheme == 'https'
    assert str(url) == 'https://example.com/'
    assert str(v.validate_python('http://other.com')) == 'http://other.com/'


def test_allowed_schemes_error():
    v = SchemaValidator(core_schema.url_schema(allowed_schemes=['http', 'https']))
    with pytest.raises(ValidationError) as exc_info:
        v.validate_python('unix:/run/foo.socket')
    # insert_assert(exc_info.value.errors())
    assert exc_info.value.errors() == [
        {
            'type': 'url_schema',
            'loc': (),
            'msg': "URL schema should be 'http' or 'https'",
            'input': 'unix:/run/foo.socket',
            'ctx': {'expected_schemas': "'http' or 'https'"},
        }
    ]


def test_allowed_schemes_errors():
    v = SchemaValidator(core_schema.url_schema(allowed_schemes=['a', 'b', 'c']))
    with pytest.raises(ValidationError) as exc_info:
        v.validate_python('unix:/run/foo.socket')
    # insert_assert(exc_info.value.errors())
    assert exc_info.value.errors() == [
        {
            'type': 'url_schema',
            'loc': (),
            'msg': "URL schema should be 'a', 'b' or 'c'",
            'input': 'unix:/run/foo.socket',
            'ctx': {'expected_schemas': "'a', 'b' or 'c'"},
        }
    ]


def test_url_query_repeat():
    v = SchemaValidator(core_schema.url_schema())
    url: Url = v.validate_python('https://example.com/foo/bar?a=1&a=2')
    assert str(url) == 'https://example.com/foo/bar?a=1&a=2'
    assert url.query_params() == [('a', '1'), ('a', '2')]
