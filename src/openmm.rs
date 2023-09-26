use std::path::Path;

use pyo3::{types::PyModule, IntoPy, Py, PyAny, PyObject, Python};

pub struct Modeller;

impl Modeller {
    pub fn new(_topology: (), _positions: ()) -> Self {
        todo!()
    }

    pub fn get_topology(&self) -> Topology {
        todo!()
    }
}

pub struct Topology;

impl Topology {
    pub fn atoms(&self) -> () {
        todo!()
    }
}

pub struct PDBFile;

impl PDBFile {
    pub fn new(_filename: impl AsRef<Path>) -> Self {
        todo!()
    }

    pub fn topology(&self) -> Topology {
        todo!();
    }

    pub fn positions(&self) {
        todo!();
    }
}

pub struct Simulation;

impl Simulation {
    pub fn context(&self) -> Context {
        todo!();
    }
}

pub struct System {
    pub(crate) inner: Py<PyAny>,
}

pub enum Integrator {
    /// time step in femtoseconds
    Verlet(f64),
}

impl IntoPy<PyObject> for Integrator {
    fn into_py(self, py: Python<'_>) -> PyObject {
        let openmm = PyModule::from_code(
            py,
            "def make_integrator(time_step):
    import openmm
    return openmm.VerletIntegrator(time_step * openmm.unit.femtoseconds)",
            "",
            "",
        )
        .unwrap()
        .getattr("make_integrator")
        .unwrap();
        match self {
            Integrator::Verlet(time_step) => {
                openmm.call1((time_step,)).unwrap().into()
            }
        }
    }
}

pub enum Platform {
    Reference,
}

impl Platform {
    pub fn by_name(_name: &str) -> Self {
        todo!();
    }
}

impl IntoPy<PyObject> for Platform {
    fn into_py(self, py: Python<'_>) -> PyObject {
        let openmm = PyModule::import(py, "openmm").unwrap();
        match self {
            Platform::Reference => {
                let platform = openmm.getattr("Platform").unwrap();
                platform
                    .call_method1("getPlatformByName", ("Reference",))
                    .unwrap()
                    .into()
            }
        }
    }
}

pub struct Context {
    inner: Py<PyAny>,
}

impl Context {
    pub fn new(
        system: System,
        integrator: Integrator,
        platform: Platform,
    ) -> Self {
        let inner = Python::with_gil(|py| {
            let openmm = PyModule::import(py, "openmm").unwrap();
            openmm
                .getattr("Context")
                .unwrap()
                .call1((system.inner, integrator, platform))
                .unwrap()
                .into()
        });
        Context { inner }
    }

    /// call `self.setPositions` with `positions` assuming input in Bohr
    pub fn set_positions(&mut self, positions: Vec<f64>) {
        let positions: Vec<[f64; 3]> = positions
            .array_chunks::<3>()
            .map(|s| s.to_owned())
            .collect();
        Python::with_gil(|py| {
            let set_positions = PyModule::from_code(
                py,
                "def set_positions(ctx, positions):
    import openmm
    positions = (positions * openmm.unit.bohr).in_units_of(openmm.unit.nanometer)
    ctx.setPositions(positions)
",
                "",
                "",
            )
            .unwrap()
            .getattr("set_positions")
            .unwrap();
            set_positions.call1((&self.inner, positions)).unwrap();
        });
    }

    /// minimize self using `LocalEnergyMinimizer`
    pub fn minimize(&mut self, f1: f64, steps: usize) {
        Python::with_gil(|py| {
            let openmm = PyModule::import(py, "openmm").unwrap();
            let m = openmm.getattr("LocalEnergyMinimizer").unwrap();
            m.call_method1("minimize", (&self.inner, f1, steps))
                .unwrap();
        });
    }

    pub fn get_coordinates(&self) -> Vec<f64> {
        Python::with_gil(|py| {
            let get_coordinates = PyModule::from_code(
                py,
                "def get_coordinates(ctx):
    import openmm
    ret = ctx.getState(getPositions=True).getPositions()\
           .value_in_unit(openmm.unit.angstrom)
    out = []
    for vec3 in ret:
        out.extend([vec3.x, vec3.y, vec3.z])
    return out
",
                "",
                "",
            )
            .unwrap()
            .getattr("get_coordinates")
            .unwrap();
            get_coordinates
                .call1((&self.inner,))
                .unwrap()
                .extract()
                .unwrap()
        })
    }

    pub fn get_energy(&self) -> f64 {
        Python::with_gil(|py| {
            let get_energy = PyModule::from_code(
                py,
                "def get_energy(ctx):
    import openmm
    ret = ctx.getState(getEnergy=True)\
           .getPotentialEnergy().value_in_unit(openmm.unit.kilocalorie_per_mole)
    return ret
",
                "",
                "",
            )
            .unwrap()
            .getattr("get_energy")
            .unwrap();
            get_energy.call1((&self.inner,)).unwrap().extract().unwrap()
        })
    }
}
