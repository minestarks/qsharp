// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_data_structures::index_map::IndexMap;

/// Provides support for qubit id allocation, measurement and
/// reset operations for Base Profile targets.
///
/// Since qubit reuse is disallowed, a mapping is maintained
/// from allocated qubit ids to hardware qubit ids. Each time
/// a qubit is reset, it is remapped to a fresh hardware qubit.
///
/// Note that even though qubit reset & reuse is disallowed,
/// qubit ids are still reused for new allocations.
/// Measurements are tracked and deferred.
#[derive(Default)]
pub struct Remapper {
    next_meas_id: usize,
    next_qubit_id: usize,
    next_qubit_hardware_id: HardwareId,
    qubit_map: IndexMap<usize, HardwareId>,
    measurements: Vec<(HardwareId, usize)>,
    // All the qubits that have ever been mapped to this hardware qubit
    // Key is HardwareId!
    // I don't actually think the same hardware qubit can be mapped to more than one qubit
    // in this layer . The opposite may be true though.
    ever_mapped: IndexMap<usize, Vec<usize>>,
}

impl Remapper {
    pub fn map(&mut self, qubit: usize) -> HardwareId {
        let m = if let Some(mapped) = self.qubit_map.get(qubit) {
            *mapped
        } else {
            let mapped = self.next_qubit_hardware_id;
            self.next_qubit_hardware_id.0 += 1;
            self.qubit_map.insert(qubit, mapped);
            mapped
        };
        let ever = if let Some(ever) = self.ever_mapped.get_mut(m.0) {
            ever
        } else {
            self.ever_mapped.insert(m.0, Vec::new());
            self.ever_mapped.get_mut(m.0).expect("yada yada")
        };
        ever.push(qubit);
        m
    }

    #[must_use]
    pub fn get_ever_mapped(&self, id: HardwareId) -> Vec<usize> {
        self.ever_mapped.get(id.0).expect("yada yada").clone()
    }

    pub fn m(&mut self, q: usize) -> usize {
        let mapped_q = self.map(q);
        let id = self.get_meas_id();
        self.measurements.push((mapped_q, id));
        id
    }

    pub fn mreset(&mut self, q: usize) -> usize {
        let id = self.m(q);
        self.reset(q);
        id
    }

    pub fn reset(&mut self, q: usize) {
        self.qubit_map.remove(q);
    }

    pub fn qubit_allocate(&mut self) -> usize {
        let id = self.next_qubit_id;
        self.next_qubit_id += 1;
        let _ = self.map(id);
        id
    }

    pub fn qubit_release(&mut self, _q: usize) {
        self.next_qubit_id -= 1;
    }

    pub fn measurements(&self) -> impl Iterator<Item = &(HardwareId, usize)> {
        self.measurements.iter()
    }

    #[must_use]
    pub fn num_allocated_qubits(&self) -> usize {
        self.next_qubit_id
    }

    #[must_use]
    pub fn num_hardware_qubits(&self) -> usize {
        self.next_qubit_hardware_id.0
    }

    #[must_use]
    pub fn num_measurements(&self) -> usize {
        self.next_meas_id
    }

    #[must_use]
    fn get_meas_id(&mut self) -> usize {
        let id = self.next_meas_id;
        self.next_meas_id += 1;
        id
    }
}

#[derive(Copy, Clone, Default)]
pub struct HardwareId(pub usize);
