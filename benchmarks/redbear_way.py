import redbear as rb
import uuid
import random
from time import perf_counter


def run(dict_size, ops_count):
    data = {str(uuid.uuid4()): random.randrange(100000) for _ in range(1, dict_size)}

    time_start = perf_counter()
    for _ in range(1, ops_count):
        x = rb.add_scalar(data, 4)
        x = rb.add(x, data)
        x = rb.subtract_scalar(x, 1)
        x = rb.subtract(x, data)
        x = rb.multiply_scalar(x, 2)
    time_duration = perf_counter() - time_start

    print(f"Redbear {ops_count} X 5 Element-wise ops on collection of {dict_size}:")
    print(f"  {time_duration:.3f} seconds")


if __name__ == "__main__":
    run(10, 100000)
    run(10, 1000000)
    # run(1000, 10000)
    # run(1000, 100000)
