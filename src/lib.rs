use std::collections::HashMap;

use pyo3::prelude::*;

/// Adds a scalar value to all the items of a dict.
#[pyfunction]
fn add_scalar(d: HashMap<String, f64>, value: f64) -> PyResult<HashMap<String, f64>> {
    let mut result: HashMap<String, f64> = HashMap::with_capacity(d.capacity());
    for (key, val) in d.iter() {
        result.insert(key.to_string(), val + value);
    }

    Ok(result)
}

/// Adds the items in a dict with the same key.
#[pyfunction]
fn add(d1: HashMap<String, f64>, d2: HashMap<String, f64>) -> PyResult<HashMap<String, f64>> {
    let mut result: HashMap<String, f64> = HashMap::with_capacity(d1.capacity());
    for (key, val) in d1.iter() {
        result.insert(key.to_string(), val + d2.get(key).unwrap_or(&0.0));
    }

    Ok(result)
}

/// Subtracts a scalar value to all the items of a dict.
#[pyfunction]
fn subtract_scalar(d: HashMap<String, f64>, value: f64) -> PyResult<HashMap<String, f64>> {
    let mut result: HashMap<String, f64> = HashMap::with_capacity(d.capacity());
    for (key, val) in d.iter() {
        result.insert(key.to_string(), val + value);
    }

    Ok(result)
}

/// Subtracts the items in a dict with the same key.
#[pyfunction]
fn subtract(d1: HashMap<String, f64>, d2: HashMap<String, f64>) -> PyResult<HashMap<String, f64>> {
    let mut result: HashMap<String, f64> = HashMap::with_capacity(d1.capacity());
    for (key, val) in d1.iter() {
        result.insert(key.to_string(), val - d2.get(key).unwrap_or(&0.0));
    }

    Ok(result)
}

/// Multiplies a scalar value to all the items of a dict.
#[pyfunction]
fn multiply_scalar(d: HashMap<String, f64>, value: f64) -> PyResult<HashMap<String, f64>> {
    let mut result: HashMap<String, f64> = HashMap::with_capacity(d.capacity());
    for (key, val) in d.iter() {
        result.insert(key.to_string(), val * value);
    }

    Ok(result)
}

/// A Python module implemented in Rust.
#[pymodule]
fn redbear(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(add_scalar, m)?)?;
    m.add_function(wrap_pyfunction!(add, m)?)?;
    m.add_function(wrap_pyfunction!(subtract_scalar, m)?)?;
    m.add_function(wrap_pyfunction!(subtract, m)?)?;
    m.add_function(wrap_pyfunction!(multiply_scalar, m)?)?;
    Ok(())
}
