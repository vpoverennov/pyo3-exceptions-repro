use pyo3::prelude::*;

#[derive(FromPyObject, Debug)]
struct Wrapper {
    n: Nested,
}

#[derive(FromPyObject, Debug)]
struct Nested {
    v: i32,
}

#[pyfunction]
fn repro_w(a: Wrapper) -> i32 {
    println!("repro_w: {:?}", a);
    a.n.v
}

#[pyfunction]
fn repro_n(b: Nested) -> i32 {
    println!("repro_n: {:?}", b);
    b.v
}

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::ffi::c_str;
    use pyo3::types::PyDict;

    fn get_module(py: Python<'_>) -> PyResult<Bound<'_, PyModule>> {
        let module = PyModule::new(py, "test_module")?;
        module.add_function(wrap_pyfunction!(repro_w, &module)?)?;
        module.add_function(wrap_pyfunction!(repro_n, &module)?)?;
        Ok(module)
    }
    #[test]
    fn test_success() {
        Python::attach(|py| {
            let code = c_str!(
                r#"
class Wrapper:
    pass

class Nested:
    @property
    def v(self):
        return 42

a = Wrapper()
a.n = Nested()

assert m.repro_w(a) == 42 # succeeds
assert m.repro_n(a.n) == 42 # succeeds
"#
            );
            let module = get_module(py).unwrap();
            let globals = PyDict::new(py);
            globals.set_item("m", &module).unwrap();

            let locals = PyDict::new(py);
            py.run(code, Some(&globals), Some(&locals)).unwrap();
        });
    }

    #[test]
    fn test_failure_wrapper() {
        Python::attach(|py| {
            let code = c_str!(
                r#"
import traceback

class Wrapper:
    pass

class Nested:
    @property
    def v(self):
        raise SystemExit(1)

a = Wrapper()
a.n = Nested()

m.repro_w(a) # raises TypeError (expected SystemExit)
"#
            );

            let module = get_module(py).unwrap();
            let globals = PyDict::new(py);
            globals.set_item("m", &module).unwrap();

            let locals = PyDict::new(py);
            let res = py.run(code, Some(&globals), Some(&locals));
            let Err(err) = res else {
                panic!("should return error");
            };
            assert_eq!(err.to_string(), "TypeError: argument 'a': failed to extract field Wrapper.n");
        });
    }

    #[test]
    fn test_failure_nested() {
        Python::attach(|py| {
            let code = c_str!(
                r#"
import traceback

class Wrapper:
    pass

class Nested:
    @property
    def v(self):
        raise SystemExit(1)

a = Wrapper()
a.n = Nested()

m.repro_n(a.n) # raises SystemExit (expected)
"#
            );

            let module = get_module(py).unwrap();
            let globals = PyDict::new(py);
            globals.set_item("m", &module).unwrap();

            let locals = PyDict::new(py);
            let res = py.run(code, Some(&globals), Some(&locals));
            let Err(err) = res else {
                panic!("should return error");
            };
            assert_eq!(err.to_string(), "SystemExit: 1");
        });
    }
}
