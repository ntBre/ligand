//! second attempt at openff-toolkit stuff, this time wrapping Python calls with
//! pyo3

use anyhow::Result;

use crate::forcefield::ForceField;

macro_rules! py_method {
    ($name:ident, $ret:ty) => {
        pub(crate) fn $name(&self) -> $ret {
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
        pub(crate) fn $name(&self) -> $ret {
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
        pub(crate) fn $name(&self, $($arg: $arg_typ)*) -> $ret {
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
        pub(crate) fn $name(&self, $($arg: $arg_typ)*) -> $ret {
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

mod molecule;
pub mod forcefield {
    use pyo3::prelude::*;

    use crate::molecule::{Labels, Topology};

    #[derive(Debug)]
    pub struct ForceField {
        inner: Py<PyAny>,
    }

    pub enum ParameterType {
        Bonds,
        Angles,
        Torsions,
    }

    impl ParameterType {
        fn as_str(&self) -> &'static str {
            match self {
                ParameterType::Bonds => "Bonds",
                ParameterType::Angles => "Angles",
                ParameterType::Torsions => "ProperTorsions",
            }
        }
    }

    pub struct ParameterHandler {
        _inner: Py<PyAny>,
    }

    // impl ParameterHandler {
    //     pub(crate) fn parameters(&self) {
    //     }
    // }

    impl ForceField {
        pub(crate) fn new(name: &str) -> anyhow::Result<Self> {
            let inner = Python::with_gil(|py| {
                let openff_toolkit = PyModule::import(py, "openff.toolkit")?;
                Ok::<_, anyhow::Error>(
                    openff_toolkit
                        .getattr("ForceField")?
                        .call1((String::from(name),))?
                        .into(),
                )
            })?;
            Ok(Self { inner })
        }

        pub fn get_parameter_handler(
            &self,
            typ: ParameterType,
        ) -> ParameterHandler {
            let inner = Python::with_gil(|py| {
                self.inner
                    .call_method1(py, "get_parameter_handler", (typ.as_str(),))
                    .unwrap()
            });
            ParameterHandler { _inner: inner }
        }

        py_method! { label_molecules, Labels, top => Topology, into }
    }
}

fn main() -> Result<()> {
    let mol = molecule::Molecule::from_mapped_smiles(
        "[Cl:2][C@:1]([F:3])([I:4])[H:5]",
    )?;
    dbg!(mol.to_inchi());
    dbg!(mol.chemical_environment_matches("[#6:1]-[#9:2]"));
    let ff = ForceField::new("openff-2.1.0.offxml")?;
    let labels = ff.label_molecules(mol.to_topology());
    dbg!(labels.get("Bonds"));
    // let h = ff.get_parameter_handler(ParameterType::Bonds);
    // dbg!(h.parameters());
    Ok(())
}
