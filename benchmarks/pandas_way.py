import pandas as pd
import uuid
import random
from time import perf_counter


def run(dict_size, ops_count):
    data = {str(uuid.uuid4()): random.randrange(100000) for _ in range(0, dict_size)}

    time_start = perf_counter()
    df = pd.Series(data, index=data.keys())
    for _ in range(0, ops_count):
        x = df.add(4)
        x = df.add(x)
        x = df.subtract(1)
        x = df.subtract(x)
        x = df.multiply(x)
    time_duration = perf_counter() - time_start


    print(f"| Pandas | {ops_count} x 5 | {dict_size} | {time_duration:.3f} |")


if __name__ == "__main__":
    run(10, 100000)
    run(10, 1000000)
    run(1000, 10000)
    run(1000, 100000)
