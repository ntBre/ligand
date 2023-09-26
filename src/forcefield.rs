use pyo3::prelude::*;
use pyo3::types::IntoPyDict;

use crate::molecule::{Labels, Topology};

use crate::openmm::{self, System};

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
    inner: Py<PyAny>,
}

pub struct Interchange {
    inner: Py<PyAny>,
}

impl Interchange {
    pub fn to_openmm(&self) -> System {
        let inner = Python::with_gil(|py| {
            self.inner.call_method0(py, "to_openmm").unwrap()
        });
        System { inner }
    }

    pub fn virtual_sites(&self) -> Vec<()> {
        todo!();
    }

    pub fn to_openmm_topology(&self) -> openmm::Topology {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct ForceField {
    inner: Py<PyAny>,
}

impl ForceField {
    pub fn new(name: &str) -> anyhow::Result<Self> {
        let inner = Python::with_gil(|py| {
            let openff_toolkit = PyModule::import(py, "openff.toolkit")?;
            let kwargs = [("allow_cosmetic_attributes", true)].into_py_dict(py);
            Ok::<_, anyhow::Error>(
                openff_toolkit
                    .getattr("ForceField")?
                    .call((String::from(name),), Some(kwargs))?
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
        ParameterHandler { inner }
    }

    pub fn create_interchange(
        &self,
        topology: Topology,
    ) -> anyhow::Result<Interchange> {
        let inner = Python::with_gil(|py| {
            self.inner
                .call_method1(py, "create_interchange", (topology.inner,))
        })?;
        Ok(Interchange { inner })
    }

    py_method! { label_molecules, Labels, top => Topology, into }

    pub fn bonds(&self) -> Vec<Bond> {
        let ph = self.get_parameter_handler(ParameterType::Bonds);
        Python::with_gil(|py| {
            let fun = PyModule::from_code(
                py,
                r#"def get_bonds(h):
    bonds = []
    for b in h:
        got = getattr(b, "_parameterize", None)
        bonds.append({"parameterize": got, "value": b.length.magnitude, "unit": "angstrom"})
    return bonds
            "#,
                "",
                "",
            )
            .unwrap()
            .getattr("get_bonds")
            .unwrap();
            fun.call1((ph.inner,)).unwrap().extract().unwrap()
        })
    }

    pub fn angles(&self) -> Vec<Angle> {
        let ph = self.get_parameter_handler(ParameterType::Angles);
        Python::with_gil(|py| {
            let fun = PyModule::from_code(
                py,
                r#"def get_angles(h):
    angles = []
    #for b in h:
    #    got = getattr(b, "_parameterize", None)
    #    angles.append({"parameterize": got, "value": b.angle.magnitude, "unit": "angstrom"})
    #print(angles)
    return angles
            "#,
                "",
                "",
            )
            .unwrap()
            .getattr("get_angles")
            .unwrap();
            fun.call1((ph.inner,)).unwrap().extract().unwrap()
        })
    }

    pub fn proper_torsions(&self) -> Vec<ProperTorsion> {
        let ph = self.get_parameter_handler(ParameterType::Torsions);
        Python::with_gil(|py| {
            let fun = PyModule::from_code(
                py,
                r#"def get_propertorsions(h):
    propertorsions = []
    for b in h:
        got = getattr(b, "_parameterize", None)
        propertorsions.append({"parameterize": got, "value": b.value.magnitude, "unit": "angstrom"})
    print(propertorsions)
    return propertorsions
            "#,
                "",
                "",
            )
            .unwrap()
            .getattr("get_propertorsions")
            .unwrap();
            fun.call1((ph.inner,)).unwrap().extract().unwrap()
        })
    }

    pub fn to_xml(&self) -> String {
        todo!();
    }
}

#[derive(FromPyObject, Clone, Debug)]
pub struct Unit(String);

#[derive(FromPyObject, Debug)]
pub struct Bond {
    #[pyo3(item)]
    pub parameterize: Option<String>,
    #[pyo3(item)]
    pub value: f64,
    #[pyo3(item)]
    pub unit: Unit,
}

#[derive(FromPyObject)]
pub struct Angle {
    #[pyo3(item)]
    pub parameterize: Option<String>,
    #[pyo3(item)]
    pub value: f64,
    #[pyo3(item)]
    pub unit: Unit,
}

#[derive(FromPyObject)]
pub struct ProperTorsion {
    #[pyo3(item)]
    pub parameterize: Option<String>,
    #[pyo3(item)]
    pub value: f64,
    #[pyo3(item)]
    pub unit: Unit,
}
