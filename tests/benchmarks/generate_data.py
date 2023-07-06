import json
from pathlib import Path
from typing import Any, Callable, List, TypeVar, Union

from faker import Faker

THIS_DIR = Path(__file__).parent


f = Faker()
Faker.seed(0)


T = TypeVar('T')

## Helper functions


def one_of(*callables: Callable[[], Any]) -> Any:
    return f.random.choice(callables)()


def list_of(callable: Callable[[], T], max_length: int) -> List[T]:
    return [callable() for _ in range(f.random_int(max=max_length))]


def lax_int(*args: Any, **kwargs: Any) -> Union[int, float, str]:
    return f.random.choice((int, float, str))(f.random_int(*args, **kwargs))


def lax_float(*args: Any, **kwargs: Any) -> Union[int, float, str]:
    return f.random.choice((int, float, str))(f.pyfloat(*args, **kwargs))


def time_seconds() -> int:
    dt = f.date_time()
    midnight = dt.replace(hour=0, minute=0, second=0, microsecond=0)
    return (dt - midnight).total_seconds()


def time_microseconds() -> float:
    return float(time_seconds()) + (f.random_int(max=999999) * 1e-6)


def time_string() -> str:
    return f.time()


def lax_time() -> Union[int, float, str]:
    return one_of(time_seconds, time_microseconds, time_string)


def datetime_timestamp() -> int:
    dt = f.date_time()
    midnight = dt.replace(hour=0, minute=0, second=0, microsecond=0)
    return (dt - midnight).total_seconds()


def datetime_microseconds() -> float:
    return float(datetime_timestamp()) + (f.random_int(max=999999) * 1e-6)


def datetime_str() -> str:
    return f.date_time().isoformat()


def lax_datetime() -> Union[int, float, str]:
    return one_of(datetime_timestamp, datetime_microseconds, datetime_str)


## Sample data generators


def blog() -> dict:
    return {
        'type': 'blog',
        'title': f.text(max_nb_chars=40),
        'post_count': lax_int(),
        'readers': lax_int(),
        'avg_post_rating': lax_float(min_value=0, max_value=5),
        'url': f.url(),
    }


def social_profile() -> dict:
    return {
        'type': 'profile',
        'username': f.user_name(),
        'join_date': f.date().format('%Y-%m-%d'),
        **one_of(facebook_profile, twitter_profile, linkedin_profile),
    }


def facebook_profile() -> dict:
    return {'network': 'facebook', 'friends': lax_int()}


def twitter_profile() -> dict:
    return {'network': 'twitter', 'followers': lax_int()}


def linkedin_profile() -> dict:
    return {'network': 'linkedin', 'connections': min(f.random_int(), 500)}


def website() -> dict:
    return one_of(blog, social_profile)


def person() -> dict:
    return {
        'name': f.name(),
        'email': f.safe_email(),
        'entry_created_date': f.date().format('%Y-%m-%d'),
        'entry_created_time': lax_time(),
        'entry_updated_at': lax_datetime(),
        'websites': list_of(website, max_length=5),
    }


if __name__ == '__main__':
    (THIS_DIR / 'sample_data.json').write_text(
        json.dumps([person() for _ in range(1000)])
        # + '\n' to make end-of-file-fixer happy
        + '\n'
    )
