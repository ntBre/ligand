//! second attempt at openff-toolkit stuff, this time wrapping Python calls with
//! pyo3

use anyhow::Result;

use crate::{forcefield::ForceField, molecule::Molecule};

#[macro_use]
mod macros;

pub mod forcefield;
pub mod molecule;

fn main() -> Result<()> {
    let mol = Molecule::from_mapped_smiles("[Cl:2][C@:1]([F:3])([I:4])[H:5]")?;
    dbg!(mol.to_inchi());
    dbg!(mol.chemical_environment_matches("[#6:1]-[#9:2]"));
    let ff = ForceField::new("openff-2.1.0.offxml")?;
    let labels = ff.label_molecules(mol.to_topology());
    dbg!(labels.get("Bonds"));
    // let h = ff.get_parameter_handler(ParameterType::Bonds);
    // dbg!(h.parameters());
    Ok(())
}
