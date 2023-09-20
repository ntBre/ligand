macro_rules! py_method {
($name:ident, $ret:ty) => {
    pub fn $name(&self) -> $ret {
        Python::with_gil(|py| {
            self.inner
                .call_method1(py, stringify!($name), ())
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }
};

($name:ident, $ret:ty, $into:ident) => {
    pub fn $name(&self) -> $ret {
        let inner = Python::with_gil(|py| {
            self.inner
                .call_method1(py, stringify!($name), ())
                .unwrap()
                .into()
        });
        <$ret>::new(inner)
    }
};

($name:ident, $ret:ty, $($arg:ident => $arg_typ:ty)*) => {
    pub fn $name(&self, $($arg: $arg_typ)*) -> $ret {
        Python::with_gil(|py| {
            self.inner
                .call_method1(py, stringify!($name), ($($arg,)*))
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }
};

($name:ident, $ret:ty, $($arg:ident => $arg_typ:ty)*, $into:ident) => {
    pub fn $name(&self, $($arg: $arg_typ)*) -> $ret {
        let inner = Python::with_gil(|py| {
            self.inner
                .call_method1(py, stringify!($name), ($($arg,)*))
                .unwrap()
                .into()
        });
        <$ret>::new(inner)
    }
};
}
