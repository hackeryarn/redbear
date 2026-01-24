import redbear as rb
import uuid
import random
from time import perf_counter


def run(dict_size, ops_count):
    data = {str(uuid.uuid4()): random.randrange(100000) for _ in range(1, dict_size)}

    time_start = perf_counter()
    d = rb.RedDict(data)
    for _ in range(1, ops_count):
        x = d.add_scalar(4)
        x = x.add(d)
        x = x.subtract_scalar(4)
        x = x.subtract(d)
        x = x.multiply(d)
    time_duration = perf_counter() - time_start

    print(f"Redbear {ops_count} X 5 Element-wise ops on collection of {dict_size}:")
    print(f"  {time_duration:.3f} seconds")


if __name__ == "__main__":
    run(10, 100000)
    run(10, 1000000)
    run(1000, 10000)
    # This runs for a very long time so it's good to leave it out.
    run(1000, 100000)
