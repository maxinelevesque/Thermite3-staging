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
}

#[derive(Debug, Clone)]
pub struct RegionIndex {
    structs: BTreeMap<String, BTreeMap<String, Type>>,
    declared_types: BTreeSet<String>,
    shared: BTreeMap<String, Type>,
}

impl RegionIndex {
    pub fn build(program: &Program) -> Result<Self, Vec<RegionError>> {
        let mut structs = BTreeMap::new();
        let mut declared_types = BTreeSet::new();
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
                _ => {}
            }
        }

        let index = Self {
            structs,
            declared_types,
            shared,
        };
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
}

pub fn effect_path(effect: &Effect) -> Option<&RegionPath> {
    match effect {
        Effect::Read(path) | Effect::Write(path) | Effect::Net(path) => Some(path),
        Effect::Alloc
        | Effect::Time
        | Effect::Rand
        | Effect::Panic
        | Effect::Diverge
        | Effect::Term => None,
    }
}
