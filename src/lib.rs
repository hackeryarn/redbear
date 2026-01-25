use std::collections::HashMap;
use std::sync::Arc;

use pyo3::{prelude::*, types::PyDict};

#[pyclass]
#[derive(Clone)]
struct RedDict {
    /// Stable key order shared across clones.
    keys: Arc<Vec<String>>,
    /// Mapping from key -> index into `values`.
    index: Arc<HashMap<String, usize>>,
    /// Packed numeric values, aligned with `keys`.
    values: Arc<Vec<f64>>,
}

#[pymethods]
impl RedDict {
    #[new]
    fn new(dict: &Bound<PyDict>) -> PyResult<Self> {
        // First extract into a temporary HashMap via PyO3, then
        // build our split key/index/value representation.
        let extracted: HashMap<String, f64> = dict.extract()?;

        let mut keys = Vec::with_capacity(extracted.len());
        let mut values = Vec::with_capacity(extracted.len());
        let mut index = HashMap::with_capacity(extracted.len());

        for (k, v) in extracted {
            let pos = keys.len();
            keys.push(k.clone());
            values.push(v);
            index.insert(k, pos);
        }

        Ok(Self {
            keys: Arc::new(keys),
            index: Arc::new(index),
            values: Arc::new(values),
        })
    }

    /// Adds a scalar value (single value) to every value in the dictionary.
    fn add_scalar(&self, value: f64) -> Self {
        let mut new = self.clone();
        Arc::make_mut(&mut new.values)
            .iter_mut()
            .for_each(|val| *val += value);
        new
    }

    fn subtract_scalar(&self, value: f64) -> Self {
        let mut new = self.clone();
        Arc::make_mut(&mut new.values)
            .iter_mut()
            .for_each(|val| *val -= value);
        new
    }

    /// Adds values (d1 + d2), aligned on d1s keys. Only keys from d1 are
    /// considered, if key from d1 is absent from d2, a fill value can optionally
    /// be used as the argument for +.
    #[pyo3(signature = (other, fill=0.0))]
    fn add(&self, other: &Bound<Self>, fill: f64) -> PyResult<Self> {
        let other_ref = other.borrow();
        Ok(merge(self, &other_ref, fill, |a, b| a + b))
    }

    fn subtract(&self, other: &Bound<Self>) -> PyResult<Self> {
        let other_ref = other.borrow();
        Ok(merge(self, &other_ref, 0.0, |a, b| a - b))
    }

    fn multiply(&self, other: &Bound<Self>) -> PyResult<Self> {
        let other_ref = other.borrow();
        Ok(merge(self, &other_ref, 0.0, |a, b| a * b))
    }

    #[getter]
    fn value(&self) -> HashMap<String, f64> {
        let mut map = HashMap::with_capacity(self.keys.len());
        for (k, v) in self.keys.iter().zip(self.values.iter()) {
            map.insert(k.clone(), *v);
        }
        map
    }
}

/// Shared implementation for binary element-wise operations.
///
/// `fill` is the value used when `other` is missing a key present in `self`.
fn merge<F>(this: &RedDict, other: &RedDict, fill: f64, f: F) -> RedDict
where
    F: Fn(f64, f64) -> f64,
{
    let mut new = this.clone();
    let new_vals = Arc::make_mut(&mut new.values);

    if Arc::ptr_eq(&this.keys, &other.keys) {
        // Fast path: identical key layout, can do a pure vector op.
        for (nv, ov) in new_vals.iter_mut().zip(other.values.iter()) {
            *nv = f(*nv, *ov);
        }
    } else {
        // General case: align on self's keys using other's index map.
        for (i, key) in this.keys.iter().enumerate() {
            let rhs = other
                .index
                .get(key)
                .map(|&j| other.values[j])
                .unwrap_or(fill);
            new_vals[i] = f(new_vals[i], rhs);
        }
    }

    new
}

/// A Python module implemented in Rust.
#[pymodule]
fn redbear(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<RedDict>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::{types::PyDict, Py, Python};

    fn make_dict(py: Python<'_>, entries: &[(&str, f64)]) -> RedDict {
        let dict = PyDict::new(py);
        for (k, v) in entries {
            dict.set_item(*k, *v).unwrap();
        }
        RedDict::new(&dict).unwrap()
    }

    #[test]
    fn scalar_operations_work() {
        Python::initialize();
        Python::attach(|py| {
            let d = make_dict(py, &[("a", 1.0), ("b", -2.0)]);

            let added = d.add_scalar(3.0);
            let added_vals = added.value();
            assert_eq!(added_vals.get("a"), Some(&4.0));
            assert_eq!(added_vals.get("b"), Some(&1.0));

            let subtracted = d.subtract_scalar(1.0);
            let sub_vals = subtracted.value();
            assert_eq!(sub_vals.get("a"), Some(&0.0));
            assert_eq!(sub_vals.get("b"), Some(&-3.0));
        });
    }

    #[test]
    fn add_and_subtract_align_on_self_keys_and_use_fill_or_zero() {
        Python::initialize();
        Python::attach(|py| {
            let left = make_dict(py, &[("a", 1.0), ("b", 2.0)]);
            let right = make_dict(py, &[("b", 10.0), ("c", 100.0)]);

            let py_left = Py::new(py, left.clone()).unwrap();
            let py_right = Py::new(py, right.clone()).unwrap();
            let _left_bound = py_left.bind(py);
            let right_bound = py_right.bind(py);

            let added = left.add(&right_bound, 5.0).unwrap();
            let add_vals = added.value();
            // "a" missing from right -> uses fill value
            assert_eq!(add_vals.get("a"), Some(&(1.0 + 5.0)));
            // "b" present in both -> uses right's value
            assert_eq!(add_vals.get("b"), Some(&(2.0 + 10.0)));
            // "c" only in right -> never appears
            assert!(!add_vals.contains_key("c"));

            let subtracted = left.subtract(&right_bound).unwrap();
            let sub_vals = subtracted.value();
            // "a" missing from right -> subtracts 0.0
            assert_eq!(sub_vals.get("a"), Some(&(1.0 - 0.0)));
            // "b" present in both -> subtracts right's value
            assert_eq!(sub_vals.get("b"), Some(&(2.0 - 10.0)));
            assert!(!sub_vals.contains_key("c"));
        });
    }

    #[test]
    fn multiply_aligns_on_self_keys_and_uses_zero_fill() {
        Python::initialize();
        Python::attach(|py| {
            let left = make_dict(py, &[("a", 2.0), ("b", 3.0)]);
            let right = make_dict(py, &[("b", 10.0), ("c", 5.0)]);

            let py_right = Py::new(py, right.clone()).unwrap();
            let right_bound = py_right.bind(py);

            let result = left.multiply(right_bound).unwrap();
            let vals = result.value();

            // "a" missing from right -> multiplied by 0.0
            assert_eq!(vals.get("a"), Some(&(2.0 * 0.0)));
            // "b" present in both -> multiplied by right's value
            assert_eq!(vals.get("b"), Some(&(3.0 * 10.0)));
            // "c" only in right -> never appears
            assert!(!vals.contains_key("c"));
        });
    }
}
