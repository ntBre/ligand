use anyhow::Result;
use pyo3::{prelude::*, types::IntoPyDict};

#[derive(Debug, Clone)]
pub struct Topology {
    pub(crate) inner: Py<PyAny>,
}

impl Topology {
    pub(crate) fn new(inner: Py<PyAny>) -> Self {
        Self { inner }
    }
}

impl IntoPy<Py<PyAny>> for Topology {
    fn into_py(self, _py: Python<'_>) -> Py<PyAny> {
        self.inner
    }
}

/// A "list" of molecule labels. as I usually call it in Python, there is only
/// one molecule, so there should only be one entry in the list. that's why
/// `get` first indexes list[0] and then the underlying dict
#[derive(Debug, Clone)]
pub struct Labels {
    pub(crate) inner: Py<PyAny>,
}

impl Labels {
    pub(crate) fn new(inner: Py<PyAny>) -> Self {
        Self { inner }
    }

    pub fn get(&self, index: &'static str) -> Py<PyAny> {
        Python::with_gil(|py| {
            self.inner
                .call_method1(py, "__getitem__", (0,))
                .unwrap()
                .call_method1(py, "__getitem__", (index,))
                .unwrap()
        })
    }
}

#[derive(Debug, Clone)]
pub struct Molecule {
    pub(crate) inner: Py<PyAny>,
}

impl Molecule {
    pub fn from_mapped_smiles(smiles: &str) -> Result<Self> {
        Self::from_pattern("from_mapped_smiles", smiles)
    }

    pub fn from_inchi(inchi: &str) -> Result<Self> {
        Self::from_pattern("from_inchi", inchi)
    }

    /// compute the RMSD between two conformers of `self` using the OpenEye
    /// toolkit. the implementation is taken from ibstore. Note that `reference`
    /// is expected to be in units of Bohr, while target is in Ångstroms
    pub fn get_rmsd(&self, reference: Vec<f64>, target: Vec<f64>) -> f64 {
        Python::with_gil(|py| {
            let fun = PyModule::from_code(
                py,
                "def get_rmsd(molecule, reference, target):
    from openeye import oechem
    from openff.units import Quantity, unit
    from openff.toolkit import Molecule
    import numpy as np
    from copy import deepcopy

    molecule = deepcopy(molecule)
    reference = np.array(reference)
    reference = np.reshape(reference, (-1, 3))

    target = np.array(target)
    target = np.reshape(target, (-1, 3))

    molecule1 = Molecule(molecule)
    q = Quantity(reference, unit.bohr).to('angstrom')
    molecule1.add_conformer(q)

    molecule2 = Molecule(molecule)
    r = Quantity(target, unit.angstrom)
    molecule2.add_conformer(r)

    ret = oechem.OERMSD(
        molecule1.to_openeye(),
        molecule2.to_openeye(),
        True,
        True,
        True,
    )
    return ret
",
                "",
                "",
            )
            .unwrap()
            .getattr("get_rmsd")
            .unwrap();
            fun.call1((&self.inner, reference, target))
                .unwrap()
                .extract()
                .unwrap()
        })
    }

    /// compute the TFD between two conformers of `self` using the OpenEye
    /// toolkit. the implementation is taken from ibstore
    pub fn get_tfd(
        &self,
        reference: Vec<f64>,
        target: Vec<f64>,
    ) -> anyhow::Result<f64> {
        Python::with_gil(|py| {
            let fun = PyModule::from_code(
                py,
                "def get_tfd(molecule, reference, target):
    from openff.toolkit import Molecule
    import numpy as np
    def _rdmol(molecule, conformer):
        from copy import deepcopy
        from openff.units import Quantity, unit

        molecule = deepcopy(molecule)
        molecule.add_conformer(
            Quantity(conformer, unit.angstrom),
        )
        return molecule.to_rdkit()

    from rdkit.Chem import TorsionFingerprints

    reference = np.array(reference)
    reference = np.reshape(reference, (-1, 3))

    target = np.array(target)
    target = np.reshape(target, (-1, 3))

    return TorsionFingerprints.GetTFDBetweenMolecules(
        _rdmol(molecule, reference),
        _rdmol(molecule, target),
    )
",
                "",
                "",
            )?
            .getattr("get_tfd")?;
            Ok(fun.call1((&self.inner, reference, target))?.extract()?)
        })
    }

    /// calls the static method Molecule.are_isomorphic, which returns
    /// `(molecules_are_isomorphic, atom_map)` and returns whether or not
    /// `molecules_are_isomorphic` (the first element of the returned tuple)
    pub fn is_isomorphic(&self, other: Self) -> bool {
        Python::with_gil(|py| {
            let openff_toolkit =
                PyModule::import(py, "openff.toolkit").unwrap();
            let molecule = openff_toolkit.getattr("Molecule").unwrap();
            molecule
                .call_method1("are_isomorphic", (&self.inner, other.inner))
                .unwrap()
                .get_item(0)
                .unwrap()
                .extract()
                .unwrap()
        })
    }

    pub fn to_mapped_smiles(&self) -> String {
        Python::with_gil(|py| {
            let kwargs = [("mapped", true)].into_py_dict(py);
            self.inner
                .call_method(py, "to_smiles", (), Some(kwargs))
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    /// return the Cartesian geometry of the `idx`th conformer of `self` as a
    /// flattened vector in units of Å
    pub fn get_conformer(&self, idx: usize) -> Vec<f64> {
        Python::with_gil(|py| {
            let fun = PyModule::from_code(
                py,
                "def get_conformer(mol, idx):
    return mol.conformers[idx].magnitude.flatten().tolist()
",
                "",
                "",
            )
            .unwrap()
            .getattr("get_conformer")
            .unwrap();
            fun.call1((&self.inner, idx)).unwrap().extract().unwrap()
        })
    }

    pub fn to_svg(&self) -> String {
        Python::with_gil(|py| {
            let fun = PyModule::from_code(
                py,
                "def draw_rdkit(mol):
    from rdkit.Chem.Draw import rdDepictor, rdMolDraw2D
    rdmol = mol.to_rdkit()
    rdDepictor.SetPreferCoordGen(True)
    rdDepictor.Compute2DCoords(rdmol)
    rdmol = rdMolDraw2D.PrepareMolForDrawing(rdmol)
    drawer = rdMolDraw2D.MolDraw2DSVG(300, 300)
    drawer.DrawMolecule(rdmol)
    drawer.FinishDrawing()
    return drawer.GetDrawingText()
",
                "",
                "",
            )
            .unwrap()
            .getattr("draw_rdkit")
            .unwrap();
            fun.call1((self.inner.clone(),)).unwrap().extract().unwrap()
        })
    }

    pub fn add_conformer(&mut self, coordinates: Vec<f64>) {
        Python::with_gil(|py| {
            let fun = PyModule::from_code(
                py,
                "def add_conformer(mol, coordinates):
    from openff.units import Quantity, unit
    import numpy as np
    a = np.array(coordinates)
    a = np.reshape(a, (-1, 3))
    c = Quantity(a, unit.angstrom)
    mol.add_conformer(c)
",
                "",
                "",
            )
            .unwrap()
            .getattr("add_conformer")
            .unwrap();
            fun.call1((&self.inner, coordinates)).unwrap();
        });
    }

    /// calls `self.to_inchi(fixed_hydrogens=True)`
    pub fn to_inchi(&self) -> String {
        Python::with_gil(|py| {
            let kwargs = [("fixed_hydrogens", true)].into_py_dict(py);
            self.inner
                .call_method(py, "to_inchi", (), Some(kwargs))
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    py_method! {to_inchikey, String}

    py_method! {chemical_environment_matches, Vec<(usize, usize)>, query => &str}

    py_method! {to_topology, Topology, into}

    /// helper method for calling the Molecule.`method` constructors with kwargs
    /// of `allow_undefined_stereo = True`
    fn from_pattern(
        method: &str,
        pattern: &str,
    ) -> std::result::Result<Molecule, anyhow::Error> {
        let inner = Python::with_gil(|py| {
            PyModule::from_code(
                py,
                r#"import logging
logging.getLogger("openff").setLevel(logging.ERROR)
    "#,
                "",
                "",
            )
            .unwrap();
            let openff_toolkit = PyModule::import(py, "openff.toolkit")?;
            let kwargs = [("allow_undefined_stereo", true)].into_py_dict(py);
            Ok::<_, anyhow::Error>(
                openff_toolkit
                    .getattr("Molecule")?
                    .call_method(
                        method,
                        (String::from(pattern),),
                        Some(kwargs),
                    )?
                    .into(),
            )
        })?;

        Ok(Self { inner })
    }
}
