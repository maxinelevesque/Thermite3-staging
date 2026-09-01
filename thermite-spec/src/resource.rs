//! RFC-11 resource provenance and contagion.
//!
//! This module deliberately stops at the type/declaration boundary. Executable
//! ownership flow is a separate consumer; callers must continue to fail closed
//! for resource-bearing bodies until that consumer has accepted them.

use std::collections::{BTreeMap, BTreeSet};

use thermite_syntax::{Item, Program, RegionPath, Span, Type, VariantShape};

/// Canonical, order-independent resource provenance for declared types.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResourceEnv {
    declared: BTreeMap<String, BTreeSet<RegionPath>>,
}

impl ResourceEnv {
    /// Build the monotone provenance fixed point and validate every declaration.
    pub fn build(program: &Program) -> Result<Self, Vec<ResourceError>> {
        let definitions = definitions(program);
        let mut declared: BTreeMap<String, BTreeSet<RegionPath>> = definitions
            .iter()
            .map(|(name, definition)| {
                let seed = definition
                    .marker
                    .as_ref()
                    .map(|regions| regions.iter().cloned().collect())
                    .unwrap_or_default();
                (name.clone(), seed)
            })
            .collect();

        loop {
            let mut changed = false;
            for (name, definition) in &definitions {
                let mut next = declared.get(name).cloned().unwrap_or_default();
                for (_, ty) in &definition.components {
                    next.extend(provenance_of_type_in(ty, &declared));
                }
                if declared.get(name) != Some(&next) {
                    declared.insert(name.clone(), next);
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }

        let env = Self { declared };
        let mut errors = Vec::new();
        for (name, definition) in &definitions {
            let mut computed = BTreeSet::new();
            let mut sources: BTreeMap<RegionPath, String> = BTreeMap::new();
            for (source, ty) in &definition.components {
                for region in env.provenance_of_type(ty) {
                    computed.insert(region.clone());
                    sources.entry(region).or_insert_with(|| source.clone());
                }
            }

            match &definition.marker {
                None if !computed.is_empty() => errors.push(ResourceError::MissingMarker {
                    declaration: name.clone(),
                    computed: computed.into_iter().collect(),
                    sources: sources.into_values().collect(),
                    span: definition.span,
                }),
                Some(declared) if declared.is_empty() && computed.is_empty() => {
                    errors.push(ResourceError::EmptyContagiousMarker {
                        declaration: name.clone(),
                        span: definition.span,
                    });
                }
                Some(explicit) if !explicit.is_empty() && !computed.is_empty() => {
                    let explicit: BTreeSet<_> = explicit.iter().cloned().collect();
                    if explicit != computed {
                        errors.push(ResourceError::ProvenanceMismatch {
                            declaration: name.clone(),
                            declared: explicit.into_iter().collect(),
                            computed: computed.into_iter().collect(),
                            sources: sources.into_values().collect(),
                            span: definition.span,
                        });
                    }
                }
                None | Some(_) => {}
            }
        }

        if errors.is_empty() {
            Ok(env)
        } else {
            Err(errors)
        }
    }

    /// The complete owned provenance of a type. References and slices are
    /// borrowed views and therefore contribute no owned disposition obligation.
    pub fn provenance_of_type(&self, ty: &Type) -> BTreeSet<RegionPath> {
        provenance_of_type_in(ty, &self.declared)
    }

    /// The canonical provenance of a declared type name.
    pub fn declared(&self, name: &str) -> Option<&BTreeSet<RegionPath>> {
        self.declared.get(name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceError {
    MissingMarker {
        declaration: String,
        computed: Vec<RegionPath>,
        sources: Vec<String>,
        span: Span,
    },
    EmptyContagiousMarker {
        declaration: String,
        span: Span,
    },
    ProvenanceMismatch {
        declaration: String,
        declared: Vec<RegionPath>,
        computed: Vec<RegionPath>,
        sources: Vec<String>,
        span: Span,
    },
}

#[derive(Debug)]
struct Definition {
    marker: Option<Vec<RegionPath>>,
    components: Vec<(String, Type)>,
    span: Span,
}

fn definitions(program: &Program) -> BTreeMap<String, Definition> {
    program
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Struct(item) => Some((
                item.name.clone(),
                Definition {
                    marker: item.resource.as_ref().map(|r| r.regions.clone()),
                    components: item
                        .fields
                        .iter()
                        .map(|field| (format!("field `{}`", field.name), field.ty.clone()))
                        .collect(),
                    span: item.span,
                },
            )),
            Item::Enum(item) => {
                let mut components = Vec::new();
                for variant in &item.variants {
                    match &variant.shape {
                        VariantShape::Unit => {}
                        VariantShape::Tuple(types) => {
                            for (index, ty) in types.iter().enumerate() {
                                components.push((
                                    format!("variant `{}` payload #{index}", variant.name),
                                    ty.clone(),
                                ));
                            }
                        }
                        VariantShape::Struct(fields) => {
                            for field in fields {
                                components.push((
                                    format!("variant `{}` field `{}`", variant.name, field.name),
                                    field.ty.clone(),
                                ));
                            }
                        }
                    }
                }
                Some((
                    item.name.clone(),
                    Definition {
                        marker: item.resource.as_ref().map(|r| r.regions.clone()),
                        components,
                        span: item.span,
                    },
                ))
            }
            _ => None,
        })
        .collect()
}

fn provenance_of_type_in(
    ty: &Type,
    declared: &BTreeMap<String, BTreeSet<RegionPath>>,
) -> BTreeSet<RegionPath> {
    match ty {
        Type::Named(name) => declared.get(name).cloned().unwrap_or_default(),
        Type::Generic { name, arg } => {
            let mut result = declared.get(name).cloned().unwrap_or_default();
            result.extend(provenance_of_type_in(arg, declared));
            result
        }
        Type::Box(inner) | Type::Vec(inner) | Type::Option(inner) => {
            provenance_of_type_in(inner, declared)
        }
        Type::Result(ok, err) | Type::Map(ok, err) => {
            let mut result = provenance_of_type_in(ok, declared);
            result.extend(provenance_of_type_in(err, declared));
            result
        }
        Type::Tuple(elements) => elements
            .iter()
            .flat_map(|element| provenance_of_type_in(element, declared))
            .collect(),
        Type::Ref { .. } | Type::Slice(_) | Type::Prim(_) | Type::Unit | Type::String => {
            BTreeSet::new()
        }
    }
}
