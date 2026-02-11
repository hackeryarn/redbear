import numpy as np
import uuid
import random
from time import perf_counter


def run(dict_size, ops_count):
    data = {str(uuid.uuid4()): random.randrange(100000) for _ in range(0, dict_size)}

    time_start = perf_counter()
    array = np.fromiter(data.values(), dtype=int, count=len(data))
    for _ in range(0, ops_count):
        x = np.add(array, 4)
        x = np.add(x, array)
        x = np.subtract(array, 1)
        x = np.subtract(x, array)
        x = np.multiply(x, array)
    time_duration = perf_counter() - time_start


    print(f"| Numpy | {ops_count} x 5 | {dict_size} | {time_duration:.3f} |")


if __name__ == "__main__":
    run(10, 100000)
    run(10, 1000000)
    run(1000, 10000)
    run(1000, 100000)
