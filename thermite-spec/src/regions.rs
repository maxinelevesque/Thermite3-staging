//! RFC-9 shared-region and concurrent-root resolution.

use std::collections::{BTreeMap, BTreeSet};

use thermite_syntax::{Effect, EffectRow, Item, Program, RegionPath, Span, Type};

/// Platform ownership of a resolved effect region. Thermite computes the
/// identity; a kernel integration such as Bulla supplies this classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionClass {
    KernelOwned,
    Ambient,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegionError {
    DuplicateShared {
        name: String,
        span: Span,
    },
    UnknownSharedType {
        name: String,
        ty: String,
        span: Span,
    },
    UnknownRegionRoot {
        path: RegionPath,
    },
    UnknownRegionField {
        path: RegionPath,
        field: String,
    },
    FieldOnNonStruct {
        path: RegionPath,
        field: String,
    },
    DuplicateComposition {
        name: String,
        span: Span,
    },
    EmptyComposition {
        name: String,
        span: Span,
    },
    DuplicateConcurrentRoot {
        composition: String,
        root: String,
    },
    UnknownConcurrentRoot {
        composition: String,
        root: String,
    },
    DuplicateLock {
        name: String,
        span: Span,
    },
    UnknownGuardedRegion {
        lock: String,
        path: RegionPath,
    },
    UnknownAfterLock {
        lock: String,
        after: String,
    },
    LockOrderCycle {
        lock: String,
    },
    OverlappingLockGuards {
        first: String,
        second: String,
    },
    GuardWithoutInvariant {
        lock: String,
        path: RegionPath,
    },
}

#[derive(Debug, Clone)]
pub struct RegionIndex {
    structs: BTreeMap<String, BTreeMap<String, Type>>,
    declared_types: BTreeSet<String>,
    shared: BTreeMap<String, Type>,
    locks: BTreeMap<String, RegionPath>,
    after: BTreeMap<String, String>,
    invariant_structs: BTreeSet<String>,
}

impl RegionIndex {
    pub fn build(program: &Program) -> Result<Self, Vec<RegionError>> {
        let mut structs = BTreeMap::new();
        let mut declared_types = BTreeSet::new();
        let mut invariant_structs = BTreeSet::new();
        for item in &program.items {
            match item {
                Item::Struct(item) => {
                    declared_types.insert(item.name.clone());
                    structs.insert(
                        item.name.clone(),
                        item.fields
                            .iter()
                            .map(|field| (field.name.clone(), field.ty.clone()))
                            .collect(),
                    );
                    if item.keeps.is_some() {
                        invariant_structs.insert(item.name.clone());
                    }
                }
                Item::Enum(item) => {
                    declared_types.insert(item.name.clone());
                }
                _ => {}
            }
        }

        let mut errors = Vec::new();
        let mut shared = BTreeMap::new();
        let mut compositions = BTreeSet::new();
        let mut lock_decls = Vec::new();
        let functions: BTreeSet<&str> = program
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Fn(function) => Some(function.name.as_str()),
                _ => None,
            })
            .collect();

        for item in &program.items {
            match item {
                Item::SharedDecl(declaration) => {
                    if shared.contains_key(&declaration.name) {
                        errors.push(RegionError::DuplicateShared {
                            name: declaration.name.clone(),
                            span: declaration.span,
                        });
                        continue;
                    }
                    if let Type::Named(name) = &declaration.ty {
                        if !declared_types.contains(name) {
                            errors.push(RegionError::UnknownSharedType {
                                name: declaration.name.clone(),
                                ty: name.clone(),
                                span: declaration.span,
                            });
                            continue;
                        }
                    }
                    shared.insert(declaration.name.clone(), declaration.ty.clone());
                }
                Item::Concurrent(composition) => {
                    if !compositions.insert(composition.name.clone()) {
                        errors.push(RegionError::DuplicateComposition {
                            name: composition.name.clone(),
                            span: composition.span,
                        });
                    }
                    if composition.roots.is_empty() {
                        errors.push(RegionError::EmptyComposition {
                            name: composition.name.clone(),
                            span: composition.span,
                        });
                    }
                    let mut roots = BTreeSet::new();
                    for root in &composition.roots {
                        if !roots.insert(root) {
                            errors.push(RegionError::DuplicateConcurrentRoot {
                                composition: composition.name.clone(),
                                root: root.clone(),
                            });
                        }
                        if !functions.contains(root.as_str()) {
                            errors.push(RegionError::UnknownConcurrentRoot {
                                composition: composition.name.clone(),
                                root: root.clone(),
                            });
                        }
                    }
                }
                Item::LockDecl(lock) => lock_decls.push(lock.clone()),
                _ => {}
            }
        }

        let mut locks = BTreeMap::new();
        let mut after = BTreeMap::new();
        for lock in &lock_decls {
            if locks
                .insert(lock.name.clone(), lock.guards.clone())
                .is_some()
            {
                errors.push(RegionError::DuplicateLock {
                    name: lock.name.clone(),
                    span: lock.span,
                });
            }
            if let Some(predecessor) = &lock.after {
                after.insert(lock.name.clone(), predecessor.clone());
            }
        }
        let index = Self {
            structs,
            declared_types,
            shared,
            locks,
            after,
            invariant_structs,
        };
        for lock in &lock_decls {
            if index.resolve(&lock.guards).is_err() {
                errors.push(RegionError::UnknownGuardedRegion {
                    lock: lock.name.clone(),
                    path: lock.guards.clone(),
                });
            }
            if index.invariant_for_region(&lock.guards).is_none() {
                errors.push(RegionError::GuardWithoutInvariant {
                    lock: lock.name.clone(),
                    path: lock.guards.clone(),
                });
            }
            if let Some(predecessor) = &lock.after {
                if !index.locks.contains_key(predecessor) {
                    errors.push(RegionError::UnknownAfterLock {
                        lock: lock.name.clone(),
                        after: predecessor.clone(),
                    });
                }
            }
            let mut seen = BTreeSet::new();
            let mut cursor = lock.name.as_str();
            while let Some(next) = index.after.get(cursor) {
                if !seen.insert(cursor.to_string()) {
                    errors.push(RegionError::LockOrderCycle {
                        lock: lock.name.clone(),
                    });
                    break;
                }
                cursor = next;
            }
        }
        for (position, first) in lock_decls.iter().enumerate() {
            for second in lock_decls.iter().skip(position + 1) {
                if first.name != second.name && index.overlaps(&first.guards, &second.guards) {
                    errors.push(RegionError::OverlappingLockGuards {
                        first: first.name.clone(),
                        second: second.name.clone(),
                    });
                }
            }
        }
        // RFC-8 effect labels may be abstract carrier names in a program with
        // no RFC-9 shared-state declarations. Once shared state is declared,
        // every state path is concrete and must resolve through that inventory.
        if !index.shared.is_empty() {
            for item in &program.items {
                let Item::Fn(function) = item else { continue };
                let EffectRow::Set(effects) = &function.contract.effects else {
                    continue;
                };
                for effect in effects {
                    if let Some(path) = effect_path(effect) {
                        if let Err(error) = index.resolve(path) {
                            errors.push(error);
                        }
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(index)
        } else {
            Err(errors)
        }
    }

    pub fn resolve(&self, path: &RegionPath) -> Result<Type, RegionError> {
        let Some(root) = path.segments.first() else {
            return Err(RegionError::UnknownRegionRoot { path: path.clone() });
        };
        let Some(mut ty) = self.shared.get(root).cloned() else {
            return Err(RegionError::UnknownRegionRoot { path: path.clone() });
        };
        for field in path.segments.iter().skip(1) {
            let Type::Named(type_name) = &ty else {
                return Err(RegionError::FieldOnNonStruct {
                    path: path.clone(),
                    field: field.clone(),
                });
            };
            let Some(fields) = self.structs.get(type_name) else {
                return Err(RegionError::FieldOnNonStruct {
                    path: path.clone(),
                    field: field.clone(),
                });
            };
            let Some(field_ty) = fields.get(field) else {
                return Err(RegionError::UnknownRegionField {
                    path: path.clone(),
                    field: field.clone(),
                });
            };
            ty = field_ty.clone();
        }
        Ok(ty)
    }

    pub fn contains(&self, outer: &RegionPath, inner: &RegionPath) -> bool {
        inner.segments.starts_with(&outer.segments)
    }

    pub fn overlaps(&self, left: &RegionPath, right: &RegionPath) -> bool {
        self.contains(left, right) || self.contains(right, left)
    }

    pub fn is_declared_type(&self, name: &str) -> bool {
        self.declared_types.contains(name)
    }

    pub fn guarded_region(&self, lock: &str) -> Option<&RegionPath> {
        self.locks.get(lock)
    }

    pub fn is_after(&self, lock: &str, predecessor: &str) -> bool {
        self.after.get(lock).is_some_and(|name| name == predecessor)
    }

    pub fn locks(&self) -> impl Iterator<Item = (&str, &RegionPath)> {
        self.locks
            .iter()
            .map(|(name, region)| (name.as_str(), region))
    }

    /// Nearest invariant-bearing struct enclosing a guarded region.
    pub fn invariant_for_region(&self, path: &RegionPath) -> Option<&str> {
        self.invariant_region(path).and_then(|region| {
            self.resolve(&region).ok().and_then(|ty| match ty {
                Type::Named(name) if self.invariant_structs.contains(&name) => {
                    self.invariant_structs.get(&name).map(String::as_str)
                }
                _ => None,
            })
        })
    }

    pub fn invariant_region(&self, path: &RegionPath) -> Option<RegionPath> {
        let root = path.segments.first()?;
        let mut ty = self.shared.get(root)?;
        let mut prefix = vec![root.clone()];
        let mut found = match ty {
            Type::Named(name) if self.invariant_structs.contains(name) => Some(prefix.clone()),
            _ => None,
        };
        for field in path.segments.iter().skip(1) {
            let Type::Named(name) = ty else { break };
            ty = self.structs.get(name)?.get(field)?;
            prefix.push(field.clone());
            if let Type::Named(name) = ty {
                if self.invariant_structs.contains(name) {
                    found = Some(prefix.clone());
                }
            }
        }
        found.map(|segments| RegionPath { segments })
    }
}

pub fn effect_path(effect: &Effect) -> Option<&RegionPath> {
    match effect {
        Effect::Read(path) | Effect::Write(path) | Effect::Net(path) => Some(path),
        Effect::Alloc
        | Effect::Time
        | Effect::Rand
        | Effect::Panic
        | Effect::Diverge
        | Effect::Term
        | Effect::Owns(_) => None,
    }
}
