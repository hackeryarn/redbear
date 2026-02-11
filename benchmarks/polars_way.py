import polars as pl
import uuid
import random
from time import perf_counter


def run(dict_size, ops_count):
    data = {str(uuid.uuid4()): random.randrange(100000) for _ in range(0, dict_size)}

    time_start = perf_counter()
    series = pl.Series(data.values())
    for _ in range(0, ops_count):
        x = series + 4 
        x = x + series
        x = x - 1
        x = x - series
        x = x * series
    time_duration = perf_counter() - time_start

    print(f"| Polars | {ops_count} x 5 | {dict_size} | {time_duration:.3f} |")


if __name__ == "__main__":
    run(10, 100000)
    run(10, 1000000)
    run(1000, 10000)
    run(1000, 100000)
