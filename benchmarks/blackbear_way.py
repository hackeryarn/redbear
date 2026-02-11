import blackbear as bb
import uuid
import random
from time import perf_counter


def run(dict_size, ops_count):
    data = {str(uuid.uuid4()): random.randrange(100000) for _ in range(1, dict_size)}

    time_start = perf_counter()
    for _ in range(1, ops_count):
        x = bb.add_scalar(data, 4)
        x = bb.add(x, data)
        x = bb.subtract_scalar(x, 1)
        x = bb.subtract(x, data)
        x = bb.multiply_scalar(x, 2)
    time_duration = perf_counter() - time_start

    print(f"| Blackbear | {ops_count} x 5 | {dict_size} | {time_duration:.3f} |")


if __name__ == "__main__":
    run(10, 100000)
    run(10, 1000000)
    run(1000, 10000)
    run(1000, 100000)
